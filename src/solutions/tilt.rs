#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use crate::clock::{verify_revision, vlf4_clock};
use crate::lsm6dsm::LSM6DSM;
use biquad::{
    Biquad as _, Coefficients, DirectForm2Transposed, Q_BUTTERWORTH_F32, ToHertz as _, Type,
};
use cortex_m::singleton;
use defmt::*;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDeviceWithConfig;
use embassy_executor::Spawner;
use embassy_stm32::Peri;
use embassy_stm32::gpio::AnyPin;
use embassy_stm32::peripherals::{DMA1_CH4, DMA1_CH5, PC10, PC11, PC12, SPI3};
use embassy_stm32::spi::{Config as SpiConfig, Spi};
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    time::Hertz,
};
use embassy_sync::signal::Signal;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::{Duration, Ticker, Timer};
use micromath::F32Ext;
use nalgebra::Vector3;

use {defmt_rtt as _, panic_probe as _};

#[path = "../clock.rs"]
mod clock;
#[path = "../lsm6dsm.rs"]
mod lsm6dsm;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(vlf4_clock());
    verify_revision(p.ADC1, p.PC4);
    info!("Hello world");

    // red led
    // high -> led on; low -> led off
    // change the default feature in Cargo.toml
    #[cfg(feature = "vlf4r1")]
    let led = Output::new(p.PB9, Level::Low, Speed::Low);
    #[cfg(feature = "vlf4r2")]
    let led = Output::new(p.PD10, Level::Low, Speed::Low);

    let fire_signal = singleton!(: Signal::<NoopRawMutex, ()> = Signal::new()).unwrap();

    #[cfg(feature = "vlf4r1")]
    let cs = p.PA15;
    #[cfg(feature = "vlf4r2")]
    let cs = p.PC13;
    spawner.spawn(
        imu_task(
            p.SPI3,
            p.PC10,
            p.PC12,
            p.PC11,
            cs.into(),
            p.DMA1_CH4,
            p.DMA1_CH5,
            fire_signal,
        )
        .unwrap(),
    );

    spawner.spawn(fire_task(led, fire_signal).unwrap());
}

#[embassy_executor::task]
async fn imu_task(
    spi: Peri<'static, SPI3>,
    sck: Peri<'static, PC10>,
    mosi: Peri<'static, PC12>,
    miso: Peri<'static, PC11>,
    cs: Peri<'static, AnyPin>,
    tx_dma: Peri<'static, DMA1_CH4>,
    rx_dma: Peri<'static, DMA1_CH5>,
    fire_signal: &'static Signal<NoopRawMutex, ()>,
) {
    let mut spi_config = SpiConfig::default();
    spi_config.frequency = Hertz(1_000_000);
    let spi =
        Mutex::<NoopRawMutex, _>::new(Spi::new(spi, sck, mosi, miso, tx_dma, rx_dma, spi_config));
    let cs = Output::new(cs, Level::High, Speed::High);
    let spi_device = SpiDeviceWithConfig::new(&spi, cs, spi_config);
    let mut imu = LSM6DSM::new(spi_device);
    imu.reset().await.unwrap();

    let mut fired = false;

    let sample_rate = 10u64;
    let mut ticker = Ticker::every(Duration::from_hz(sample_rate));

    let angle_low_pass_coeff = Coefficients::<f32>::from_params(
        Type::LowPass,
        (sample_rate as f32).hz(),
        2f32.hz(),
        Q_BUTTERWORTH_F32,
    )
    .unwrap();
    let mut angle_low_pass = DirectForm2Transposed::new(angle_low_pass_coeff);
    loop {
        let measurements = imu.read().await.unwrap();

        let acc = Vector3::from_column_slice(&measurements.acc);
        let down = Vector3::new(-1f32, 0f32, 0f32);

        let angle = acc.angle(&down).to_degrees();
        let low_passed_angle = angle_low_pass.run(angle);
        info!(
            "angle: {} / {} degrees",
            (angle * 10.0).round() / 10.0,
            (low_passed_angle * 10.0).round() / 10.0
        );

        if low_passed_angle > 45.0 && !fired {
            fire_signal.signal(());
            fired = true;
        }
        ticker.next().await;
    }
}

#[embassy_executor::task]
async fn fire_task(mut led: Output<'static>, fire_signal: &'static Signal<NoopRawMutex, ()>) {
    loop {
        fire_signal.wait().await;
        led.set_high();
        Timer::after_millis(200).await;
        led.set_low();
    }
}

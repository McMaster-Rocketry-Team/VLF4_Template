#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use crate::clock::vlf4_clock;
use crate::lsm6dsm::LSM6DSM;
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
use micromath::F32Ext;

use {defmt_rtt as _, panic_probe as _};

mod clock;
mod lsm6dsm;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(vlf4_clock());
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

    info!("{}", imu.read().await);
}

#[embassy_executor::task]
async fn fire_task(mut led: Output<'static>, fire_signal: &'static Signal<NoopRawMutex, ()>) {

}
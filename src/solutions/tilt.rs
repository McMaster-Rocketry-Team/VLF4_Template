#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use crate::clock::vlf4_clock;
use crate::lsm6dsm::LSM6DSM;
use defmt::*;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDeviceWithConfig;
use embassy_executor::Spawner;
use embassy_stm32::spi::{Config as SpiConfig, Spi};
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    time::Hertz,
};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::{Duration, Ticker};

use {defmt_rtt as _, panic_probe as _};

#[path = "../clock.rs"]
mod clock;
#[path = "../lsm6dsm.rs"]
mod lsm6dsm;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(vlf4_clock());
    info!("Hello world");

    // red led
    // high -> led on; low -> led off
    // VLF4r1: ??     VLF4r2: p.PD10
    let mut led = Output::new(p.PD10, Level::Low, Speed::Low);

    let mut spi_config = SpiConfig::default();
    spi_config.frequency = Hertz(1_000_000);
    let spi = Mutex::<NoopRawMutex, _>::new(Spi::new(
        p.SPI3, p.PC10, p.PC12, p.PC11, p.DMA1_CH4, p.DMA1_CH5, spi_config,
    ));
    let low_g_imu_spi_device = SpiDeviceWithConfig::new(
        &spi,
        Output::new(p.PC13, Level::High, Speed::High),
        spi_config,
    );
    let mut imu = LSM6DSM::new(low_g_imu_spi_device);

    imu.reset().await.unwrap();

    let mut ticker = Ticker::every(Duration::from_hz(10));
    loop {
        let measurements = imu.read().await.unwrap();
        info!("{}", measurements);
        ticker.next().await;
    }
}

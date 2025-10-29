#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use crate::clock::vlf4_clock;
use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_futures::select::select;
use embassy_stm32::{
    Config,
    exti::ExtiInput,
    gpio::{Level, Output, Pull, Speed},
};
use embassy_time::Timer;

use {defmt_rtt as _, panic_probe as _};

mod clock;
mod lsm6dsm;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(vlf4_clock());
    info!("Hello world");

    // red led
    // high -> led on; low -> led off
    // VLF4r1: ??     VLF4r2: p.PD10
    let mut led = Output::new(p.PD10, Level::Low, Speed::Low);
}

#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use crate::clock::{verify_revision, vlf4_clock};
use cortex_m::singleton;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::Pull;
use embassy_stm32::gpio::{Level, Output, Speed};
#[cfg(feature = "vlf4r1")]
use embassy_stm32::peripherals::{EXTI5, PA0, PA1, PB5, UART4};
#[cfg(feature = "vlf4r2")]
use embassy_stm32::peripherals::{EXTI12, PA2, PA3, PD12, USART2};
use embassy_stm32::usart::{BufferedUart, Config as UartConfig};
use embassy_stm32::{Peri, bind_interrupts, usart};
use embedded_io_async::Read;

use {defmt_rtt as _, panic_probe as _};

mod clock;

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

    #[cfg(feature = "vlf4r1")]
    spawner.spawn(nmea_task(p.UART4, p.PA1, p.PA0).unwrap());
    #[cfg(feature = "vlf4r2")]
    spawner.spawn(nmea_task(p.USART2, p.PA3, p.PA2, nmea_unix_time_signal).unwrap());

    #[cfg(feature = "vlf4r1")]
    spawner.spawn(pps_task(led, p.PB5, p.EXTI5).unwrap());
    #[cfg(feature = "vlf4r2")]
    spawner.spawn(pps_task(led, p.PD12, p.EXTI12, nmea_unix_time_signal).unwrap());
}

#[embassy_executor::task]
async fn nmea_task(
    #[cfg(feature = "vlf4r1")] usart: Peri<'static, UART4>,
    #[cfg(feature = "vlf4r2")] usart: Peri<'static, USART2>,

    #[cfg(feature = "vlf4r1")] rx: Peri<'static, PA1>,
    #[cfg(feature = "vlf4r2")] rx: Peri<'static, PA3>,

    #[cfg(feature = "vlf4r1")] tx: Peri<'static, PA0>,
    #[cfg(feature = "vlf4r2")] tx: Peri<'static, PA2>,
) {
    #[cfg(feature = "vlf4r1")]
    bind_interrupts!(struct Irqs {
        UART4 => usart::BufferedInterruptHandler<UART4>;
    });
    #[cfg(feature = "vlf4r2")]
    bind_interrupts!(struct Irqs {
        USART2 => usart::BufferedInterruptHandler<USART2>;
    });

    let tx_buf = singleton!(: [u8; 64] = [0; 64]).unwrap();
    let rx_buf = singleton!(: [u8; 64] = [0; 64]).unwrap();
    let mut config = UartConfig::default();
    config.baudrate = 9600;
    let mut uart = BufferedUart::new(usart, rx, tx, tx_buf, rx_buf, Irqs, config).unwrap();

    let mut buffer = [0; 64];
    uart.read(&mut buffer).await;
}

#[embassy_executor::task]
async fn pps_task(
    mut led: Output<'static>,

    #[cfg(feature = "vlf4r1")] pin: Peri<'static, PB5>,
    #[cfg(feature = "vlf4r2")] pin: Peri<'static, PD12>,

    #[cfg(feature = "vlf4r1")] exti: Peri<'static, EXTI5>,
    #[cfg(feature = "vlf4r2")] exti: Peri<'static, EXTI12>,
) {
    let mut pps = ExtiInput::new(pin, exti, Pull::None);
}

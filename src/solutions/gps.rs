#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use crate::clock::{verify_revision, vlf4_clock};
use chrono::{TimeZone as _, Utc};
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
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant, Timer};
use embedded_io_async::Read;
use heapless::String;
use nmea::Nmea;
use nmea::sentences::FixType;

use {defmt_rtt as _, panic_probe as _};

#[path = "../clock.rs"]
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

    let nmea_unix_time_signal =
        singleton!(: Signal::<NoopRawMutex, (Instant, i64)> = Signal::new()).unwrap();

    #[cfg(feature = "vlf4r1")]
    spawner.spawn(nmea_task(p.UART4, p.PA1, p.PA0, nmea_unix_time_signal).unwrap());
    #[cfg(feature = "vlf4r2")]
    spawner.spawn(nmea_task(p.USART2, p.PA3, p.PA2, nmea_unix_time_signal).unwrap());

    #[cfg(feature = "vlf4r1")]
    spawner.spawn(pps_task(led, p.PB5, p.EXTI5, nmea_unix_time_signal).unwrap());
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

    nmea_unix_time_signal: &'static Signal<NoopRawMutex, (Instant, i64)>,
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
    let mut sentence = String::<84>::new();
    let mut nmea = Nmea::default();

    loop {
        match uart.read(&mut buffer).await {
            Ok(length) => {
                for i in 0..length {
                    if buffer[i] == b'$' {
                        sentence.clear();
                    }
                    sentence.push(buffer[i] as char).ok();

                    if buffer[i] == b'\n' || sentence.len() == sentence.capacity() {
                        let parse_result = nmea.parse(sentence.as_str());

                        if let Err(e) = parse_result {
                            warn!(
                                "Parse error: {:?}, sentence: {}",
                                Debug2Format(&e),
                                sentence.as_str()
                            );
                        } else {
                            info!("Parsed: {}", sentence.as_str());
                        }

                        if nmea.fix_type != None
                            && nmea.fix_type != Some(FixType::Invalid)
                            && let Some(date) = nmea.fix_date
                            && let Some(time) = nmea.fix_time
                        {
                            let datetime = date.and_time(time);
                            let datetime = Utc.from_utc_datetime(&datetime);
                            nmea_unix_time_signal.signal((Instant::now(), datetime.timestamp()));
                        }

                        sentence.clear();
                        for j in (i + 1)..length {
                            sentence.push(buffer[j] as char).ok();
                        }
                    }
                }
            }
            Err(e) => {
                error!("Error reading from UART: {}", e);
            }
        }
    }
}

#[embassy_executor::task]
async fn pps_task(
    mut led: Output<'static>,

    #[cfg(feature = "vlf4r1")] pin: Peri<'static, PB5>,
    #[cfg(feature = "vlf4r2")] pin: Peri<'static, PD12>,

    #[cfg(feature = "vlf4r1")] exti: Peri<'static, EXTI5>,
    #[cfg(feature = "vlf4r2")] exti: Peri<'static, EXTI12>,

    nmea_unix_time_signal: &'static Signal<NoopRawMutex, (Instant, i64)>,
) {
    let mut pps = ExtiInput::new(pin, exti, Pull::None);

    loop {
        pps.wait_for_rising_edge().await;
        if let Some((instant, unix_time)) = nmea_unix_time_signal.try_take()
            && Instant::now() - instant < Duration::from_millis(800)
        {
            info!("Unix timestamp: {}", unix_time + 1);
            led.set_high();
            Timer::after_millis(200).await;
            led.set_low();
        }
    }
}

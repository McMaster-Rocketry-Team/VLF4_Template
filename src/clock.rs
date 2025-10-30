use embassy_stm32::{Config, Peri};
use embassy_stm32::adc::{Adc, AdcChannel, SampleTime};
use embassy_stm32::peripherals::{ADC1, PC4};
use embassy_stm32::rcc::mux::*;
use embassy_stm32::rcc::*;

pub fn vlf4_clock() -> Config {
    let mut config = Config::default();
    config.rcc.hsi = Some(HSIPrescaler::DIV4);
    config.rcc.hse = None;
    config.rcc.csi = false;
    config.rcc.hsi48 = Some(Hsi48Config {
        sync_from_usb: false,
    });
    config.rcc.ls = LsConfig::default_lsi();

    config.rcc.pll1 = Some(Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV1,
        mul: PllMul::MUL32,
        divp: Some(PllDiv::DIV1),
        divq: Some(PllDiv::DIV4),
        divr: Some(PllDiv::DIV2),
    });

    config.rcc.pll2 = Some(Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV1,
        mul: PllMul::MUL20,
        divp: Some(PllDiv::DIV8),
        divq: Some(PllDiv::DIV2),
        divr: Some(PllDiv::DIV2),
    });
    config.rcc.pll3 = Some(Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV1,
        mul: PllMul::MUL24,
        divp: Some(PllDiv::DIV2),
        divq: Some(PllDiv::DIV8),
        divr: Some(PllDiv::DIV2),
    });

    config.rcc.sys = Sysclk::PLL1_P;
    config.rcc.d1c_pre = AHBPrescaler::DIV1;

    config.rcc.apb1_pre = APBPrescaler::DIV2;
    config.rcc.apb2_pre = APBPrescaler::DIV2;
    config.rcc.apb3_pre = APBPrescaler::DIV2;
    config.rcc.apb4_pre = APBPrescaler::DIV2;
    config.rcc.ahb_pre = AHBPrescaler::DIV2;

    config.rcc.voltage_scale = VoltageScale::Scale0;

    config.rcc.mux.spi123sel = Saisel::PLL1_Q;
    config.rcc.mux.usart234578sel = Usart234578sel::PCLK1;
    config.rcc.mux.rngsel = Rngsel::HSI48;
    config.rcc.mux.i2c4sel = I2c4sel::PCLK4;
    config.rcc.mux.i2c1235sel = I2c1235sel::PCLK1;
    config.rcc.mux.spi6sel = Spi6sel::PCLK4;
    config.rcc.mux.spi45sel = Spi45sel::PCLK2;
    config.rcc.mux.adcsel = Adcsel::PLL2_P;
    config.rcc.mux.fdcansel = Fdcansel::PLL1_Q;
    config.rcc.mux.usbsel = Usbsel::PLL3_Q;

    config
}

pub fn verify_revision(adc1: Peri<'_, ADC1>, pc4: Peri<'_, PC4>) {
    // On r1, PC4 is connected to curr_ref so it should be 2.5V
    // On r2, PC4 is connected to green led, so it should be 0V

    let mut adc = Adc::new(adc1);
    adc.set_sample_time(SampleTime::CYCLES387_5);
    let raw_value = adc.blocking_read(&mut pc4.degrade_adc());

    if raw_value > 20000 {
        // is r1
        if cfg!(feature = "vlf4r2") {
            defmt::panic!("\"vlf4r2\" feature is selected but running on r1")
        }
    }else {
        // is r2
        if cfg!(feature = "vlf4r1") {
            defmt::panic!("\"vlf4r1\" feature is selected but running on r2")
        }
    }
}

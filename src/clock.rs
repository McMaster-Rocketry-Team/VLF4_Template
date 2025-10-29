use embassy_stm32::Config;
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

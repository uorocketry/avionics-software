//! Setup and configuration of the HAL.
use embassy_stm32::{
	Config, Peripherals, init,
	rcc::{AHBPrescaler, APBPrescaler, HSIPrescaler, Pll, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk, VoltageScale},
};

pub fn configure_hal() -> Peripherals {
	let mut config = Config::default();

	// Clock configuration
	//
	config.rcc.hsi = Some(HSIPrescaler::DIV1);
	config.rcc.csi = true;

	config.rcc.pll1 = Some(Pll {
		source: PllSource::HSI,
		prediv: PllPreDiv::DIV4,
		mul: PllMul::MUL50,
		divp: Some(PllDiv::DIV2),
		divq: Some(PllDiv::DIV8),
		divr: None,
	});
	config.rcc.pll2 = Some(Pll {
		source: PllSource::HSI,
		prediv: PllPreDiv::DIV4,
		mul: PllMul::MUL50,
		divp: Some(PllDiv::DIV2),
		divq: Some(PllDiv::DIV8),
		divr: None,
	});
	config.rcc.pll3 = Some(Pll {
		source: PllSource::HSI,
		prediv: PllPreDiv::DIV4,
		mul: PllMul::MUL50,
		divp: Some(PllDiv::DIV2),
		divq: Some(PllDiv::DIV8),
		divr: None,
	});
	config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
	config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
	config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
	config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
	config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
	config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
	config.rcc.voltage_scale = VoltageScale::Scale1;

	init(config)
}

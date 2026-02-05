use core::clone;

use defmt::info;
use embassy_stm32::Peripheral;
use embassy_stm32::gpio::{Level, Pin};
use embassy_stm32::pac::octospi::regs::Ar;
use peripheral_services::gpio::peripheral::GPIOPin;

use crate::ejection_channel_driver::utils::EjectionChannelStates;
use crate::ejection_channel_driver::utils::EjectionChannelStates::*;

pub struct EjectionChannelDriver<'a> {
	trigger: GPIOPin<'a>,
	arm: GPIOPin<'a>,
	sense: GPIOPin<'a>,
	// Detected the charge detection LED (not present on old revision of phoenix)
	detected: Option<GPIOPin<'a>>,
	state: EjectionChannelStates,
}

impl<'a> EjectionChannelDriver<'a> {
	pub fn new(
		trigger: impl Peripheral<P = impl Pin> + 'a,
		arm: impl Peripheral<P = impl Pin> + 'a,
		sense: impl Peripheral<P = impl Pin> + 'a,
		detected: Option<impl Peripheral<P = impl Pin> + 'a>,
	) -> Self {
		let mut trigger = GPIOPin::new(
			trigger,
			peripheral_services::gpio::utils::GPIOMode::Output(embassy_stm32::gpio::Speed::Medium),
		);
		let mut arm = GPIOPin::new(
			arm,
			peripheral_services::gpio::utils::GPIOMode::Output(embassy_stm32::gpio::Speed::Medium),
		);
		trigger.set_low();
		arm.set_low();
		let sense: GPIOPin<'_> = GPIOPin::new(sense, peripheral_services::gpio::utils::GPIOMode::Input(embassy_stm32::gpio::Pull::Down));
		let detected_pin: Option<GPIOPin<'a>> = None;

		if let Some(pin) = detected {
			let detected_pin = Some(GPIOPin::new(
				pin,
				peripheral_services::gpio::utils::GPIOMode::Output(embassy_stm32::gpio::Speed::Medium),
			));
		}

		let mut driver = EjectionChannelDriver {
			trigger: trigger,
			arm: arm,
			sense: sense,
			detected: detected_pin,
			state: NoContinuity,
		};

		if driver.check_continuity() {
			driver.state = Idle;
		}
		driver
	}

	pub fn arm(&mut self) {
		self.arm.set_high();
		self.state = Armed;
	}

	pub fn check_continuity(&mut self) -> bool {
		match self.sense.get_logic_level() {
			embassy_stm32::gpio::Level::Low => true,
			embassy_stm32::gpio::Level::High => false,
		}
	}

	pub fn deploy_charge(&mut self) {
		self.trigger.set_high();
		self.state = Deployed;
		if let Some(detect) = &mut self.detected {
			detect.set_high();
		}
	}

	pub fn get_state(&mut self) -> EjectionChannelStates {
		self.state.clone()
	}

	// Update the state machine
	pub fn update(&mut self) {
		// TODO: Try and reduce the heavy nesting for readability
		if !self.check_continuity() {
			if self.state == Deployed {
				self.state = ConfirmedDeployed;
			} else if (self.state == Armed) {
				self.state = ContinuityLost;
			}
		}
		if self.check_continuity() {
			if self.state == ContinuityLost || self.state == NoContinuity {
				if self.arm.get_set_level() == Level::High {
					self.state = Armed
				} else {
					self.state = Idle;
				}
			}
		}
	}
}

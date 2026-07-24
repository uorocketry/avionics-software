use core::clone;

use defmt::{info, warn};
use embassy_stm32::Peripheral;
use embassy_stm32::gpio::{Level, Pin};
use embassy_stm32::pac::octospi::regs::Ar;
use uor_peripherals::gpio::peripheral::GPIOPin;

use crate::ejection_channel::utils::EjectionChannelStates;
use crate::ejection_channel::utils::EjectionChannelStates::*;

pub struct EjectionChannel<'a> {
	trigger: GPIOPin<'a>,
	arm: GPIOPin<'a>,
	sense: GPIOPin<'a>,
	// Detected the charge detection LED (not present on old revision of phoenix)
	detected: Option<GPIOPin<'a>>,
	state: EjectionChannelStates,
	armed_and_continuity: bool,
	// This may seem redundant due to internal state machine, HOWEVER, if the continuity check ever fails the rocket will never leave "NoContinuity" resulting in the charge detonation never activating (even if the check fails due to poor solder joints, or flakey wiring)
	has_been_armed: bool,
}

impl<'a> EjectionChannel<'a> {
	pub fn new(
		trigger: impl Peripheral<P = impl Pin> + 'a,
		arm: impl Peripheral<P = impl Pin> + 'a,
		sense: impl Peripheral<P = impl Pin> + 'a,
		detected: Option<impl Peripheral<P = impl Pin> + 'a>,
	) -> Self {
		let mut trigger = GPIOPin::new(
			trigger,
			uor_peripherals::gpio::utils::GPIOMode::Output(embassy_stm32::gpio::Speed::Medium),
		);
		let mut arm = GPIOPin::new(arm, uor_peripherals::gpio::utils::GPIOMode::Output(embassy_stm32::gpio::Speed::Medium));
		trigger.set_low();
		arm.set_low();
		let sense: GPIOPin<'_> = GPIOPin::new(sense, uor_peripherals::gpio::utils::GPIOMode::Input(embassy_stm32::gpio::Pull::Down));
		let detected_pin: Option<GPIOPin<'a>> = None;

		if let Some(pin) = detected {
			let detected_pin = Some(GPIOPin::new(
				pin,
				uor_peripherals::gpio::utils::GPIOMode::Output(embassy_stm32::gpio::Speed::Medium),
			));
		}

		let mut driver = EjectionChannel {
			trigger: trigger,
			arm: arm,
			sense: sense,
			detected: detected_pin,
			state: Unknown,
			armed_and_continuity: false,
			has_been_armed: false,
		};

		driver
	}

	pub fn arm(&mut self) {
		self.arm.set_high();
		self.state = Armed;
		self.has_been_armed = true;
	}

	pub fn check_continuity(&mut self) -> bool {
		match self.sense.get_logic_level() {
			embassy_stm32::gpio::Level::Low => true,
			embassy_stm32::gpio::Level::High => false,
		}
	}

	pub fn deploy_charge(&mut self) {
		if self.has_been_armed == true {
			self.trigger.set_high();
			self.state = Deployed;
		} else {
			warn!("Did not deploy as channel was never armed");
		}
	}

	pub fn get_state(&mut self) -> EjectionChannelStates {
		self.state.clone()
	}

	// Update the state machine
	pub fn update(&mut self) {
		if !self.check_continuity() {
			// Set continuity LED low
			if let Some(detect) = &mut self.detected {
				detect.set_low();
			}
			if self.state == Deployed && self.armed_and_continuity {
				self.state = ConfirmedDeployed;
			} else if self.state == Armed && !self.armed_and_continuity {
				// If arm is "fresh", then we know continuity has not been lost as it never existed to begin with
				self.state = NoContinuity;
			} else if self.state == Armed && self.armed_and_continuity {
				self.state = ContinuityLost;
			}
		} else {
			// Set continuity LED high
			if let Some(detect) = &mut self.detected {
				detect.set_high();
			}
			if self.arm.get_set_level() == Level::High && (self.state != Deployed && self.state != ConfirmedDeployed) {
				self.state = Armed;
				self.armed_and_continuity = true;
			}
		}
	}
}

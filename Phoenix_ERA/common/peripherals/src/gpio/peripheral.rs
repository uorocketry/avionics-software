use embassy_stm32::{
	Peripheral,
	gpio::{Flex, Level, Pin},
};

use crate::gpio::utils::GPIOMode;

// Although a GPIO wrapper is not needed functionality-wise, it will help down the line with simulation
pub struct GPIOPin<'a> {
	pin: Flex<'a>,
}

// TODO: Check if pin needs to be configured as input, output, etc before set_high() or get_logic_level() are called
impl<'a> GPIOPin<'a> {
	pub fn new(
		pin: impl Peripheral<P = impl Pin> + 'a,
		mode: GPIOMode,
	) -> Self {
		let pin = Flex::new(pin);
		let mut pin = GPIOPin { pin: pin };
		pin.configure(mode);
		pin
	}

	pub fn configure(
		&mut self,
		mode: GPIOMode,
	) {
		match mode {
			GPIOMode::Input(pull) => self.pin.set_as_input(pull),
			GPIOMode::Output(speed) => {
				// Make sure pin is set to low when configured
				self.pin.set_low();
				self.pin.set_as_output(speed)
			}
			GPIOMode::InputOutput(speed) => self.pin.set_as_input_output(speed),
			GPIOMode::InputOutputPull(speed, pull) => self.pin.set_as_input_output_pull(speed, pull),
		};
	}

	pub fn set_high(&mut self) {
		self.pin.set_high();
	}

	pub fn set_low(&mut self) {
		self.pin.set_low();
	}

	pub fn get_logic_level(&self) -> Level {
		self.pin.get_level()
	}

	pub fn get_set_level(&self) -> Level {
		self.pin.get_level()
	}
}

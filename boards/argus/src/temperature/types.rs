use defmt::Format;

use crate::adc::driver::types::AnalogChannel;

// Represents a single temperature reading from a thermocouple channel
#[derive(Debug, Clone, Copy, Format)]
pub struct ThermocoupleReading {
	// Timestamp of the reading in milliseconds since epoch
	pub timestamp: u64,

	// Thermocouple voltage difference measured in millivolts
	pub voltage: f32,

	// Cold-junction-compensated temperature of the thermocouple in degrees Celsius
	pub compensated_temperature: f32,

	// Uncompensated temperature of the thermocouple in degrees Celsius
	pub uncompensated_temperature: f32,

	// Temperature of the cold junction in degrees Celsius
	pub cold_junction_temperature: f32,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Thermocouple {
	Channel1 = 0,
	Channel2 = 1,
	Channel3 = 2,
	Channel4 = 3,
}

impl Thermocouple {
	pub fn from(value: usize) -> Self {
		match value {
			0 => Thermocouple::Channel1,
			1 => Thermocouple::Channel2,
			2 => Thermocouple::Channel3,
			3 => Thermocouple::Channel4,
			_ => panic!("Invalid thermocouple channel index: {}", value),
		}
	}

	pub fn to_analog_input_channel_pair(&self) -> (AnalogChannel, AnalogChannel) {
		match self {
			Thermocouple::Channel1 => (AnalogChannel::AIN0, AnalogChannel::AIN1),
			Thermocouple::Channel2 => (AnalogChannel::AIN2, AnalogChannel::AIN3),
			Thermocouple::Channel3 => (AnalogChannel::AIN4, AnalogChannel::AIN5),
			Thermocouple::Channel4 => (AnalogChannel::AIN6, AnalogChannel::AIN7),
		}
	}
}

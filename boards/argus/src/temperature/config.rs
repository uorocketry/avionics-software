// Configurations that concern the temperature measurement system
use defmt::Format;
use serde::Serialize;

use crate::adc::driver::types::AnalogChannel;

// Number of thermocouple channels per ADC
// Note: Not to get confused with the number of analog input channels on each ADC
// Each thermocouple channel uses a pair of analog input channels (differential measurement)
pub const CHANNEL_COUNT: usize = 4;

pub const QUEUE_SIZE: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Format, Serialize)]
pub enum ThermocoupleChannel {
	Channel1 = 0,
	Channel2 = 1,
	Channel3 = 2,
	Channel4 = 3,
}

// Support for implicit conversion from usize to ThermocoupleChannel
impl From<usize> for ThermocoupleChannel {
	fn from(value: usize) -> Self {
		match value {
			0 => ThermocoupleChannel::Channel1,
			1 => ThermocoupleChannel::Channel2,
			2 => ThermocoupleChannel::Channel3,
			3 => ThermocoupleChannel::Channel4,
			_ => panic!("Invalid thermocouple channel index: {}", value),
		}
	}
}

// Configure which analog input channel pair each thermocouple channel uses
impl ThermocoupleChannel {
	pub fn to_analog_input_channel_pair(&self) -> (AnalogChannel, AnalogChannel) {
		match self {
			ThermocoupleChannel::Channel1 => (AnalogChannel::AIN0, AnalogChannel::AIN1),
			ThermocoupleChannel::Channel2 => (AnalogChannel::AIN2, AnalogChannel::AIN3),
			ThermocoupleChannel::Channel3 => (AnalogChannel::AIN4, AnalogChannel::AIN5),
			ThermocoupleChannel::Channel4 => (AnalogChannel::AIN6, AnalogChannel::AIN7),
		}
	}
}

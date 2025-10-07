use defmt::Format;
use serde::{Deserialize, Serialize};

use crate::adc::driver::types::AnalogChannel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Format, Serialize, Deserialize)]
pub enum PressureChannel {
	Channel1 = 0,
	Channel2 = 1,
	Channel3 = 2,
	Channel4 = 3,
}

impl Default for PressureChannel {
	fn default() -> Self {
		PressureChannel::Channel1
	}
}

// Support for implicit conversion from usize to PressureChannel
impl From<usize> for PressureChannel {
	fn from(value: usize) -> Self {
		match value {
			0 => PressureChannel::Channel1,
			1 => PressureChannel::Channel2,
			2 => PressureChannel::Channel3,
			3 => PressureChannel::Channel4,
			_ => panic!("Invalid pressure channel index: {}", value),
		}
	}
}

// Configure which analog input channel pair each pressure channel uses
impl PressureChannel {
	pub fn to_analog_input_channel_pair(&self) -> (AnalogChannel, AnalogChannel) {
		match self {
			PressureChannel::Channel1 => (AnalogChannel::AIN0, AnalogChannel::AIN1),
			PressureChannel::Channel2 => (AnalogChannel::AIN2, AnalogChannel::AIN3),
			PressureChannel::Channel3 => (AnalogChannel::AIN4, AnalogChannel::AIN5),
			PressureChannel::Channel4 => (AnalogChannel::AIN6, AnalogChannel::AIN7),
		}
	}
}

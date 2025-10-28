use defmt::Format;
use messages::argus::strain::strain_channel::StrainChannel as StrainChannelProtobuf;
use serde::{Deserialize, Serialize};
use strum::EnumCount;

use crate::adc::driver::types::AnalogChannel;

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Format, Serialize, Deserialize, EnumCount, Default)]
pub enum StrainChannel {
	#[default]
	Channel1 = 0,
	Channel2 = 1,
	Channel3 = 2,
	Channel4 = 3,
}

// Support for implicit conversion from usize to StrainChannel
impl From<usize> for StrainChannel {
	fn from(value: usize) -> Self {
		match value {
			0 => StrainChannel::Channel1,
			1 => StrainChannel::Channel2,
			2 => StrainChannel::Channel3,
			3 => StrainChannel::Channel4,
			_ => panic!("Invalid strain channel index: {}", value),
		}
	}
}

// Configure which analog input channel pair each strain channel uses
impl StrainChannel {
	pub fn to_analog_input_channel_pair(&self) -> (AnalogChannel, AnalogChannel) {
		match self {
			StrainChannel::Channel1 => (AnalogChannel::AIN0, AnalogChannel::AIN1),
			StrainChannel::Channel2 => (AnalogChannel::AIN2, AnalogChannel::AIN3),
			StrainChannel::Channel3 => (AnalogChannel::AIN4, AnalogChannel::AIN5),
			StrainChannel::Channel4 => (AnalogChannel::AIN6, AnalogChannel::AIN7),
		}
	}

	pub fn to_protobuf(&self) -> StrainChannelProtobuf {
		match self {
			StrainChannel::Channel1 => StrainChannelProtobuf::Channel1,
			StrainChannel::Channel2 => StrainChannelProtobuf::Channel2,
			StrainChannel::Channel3 => StrainChannelProtobuf::Channel3,
			StrainChannel::Channel4 => StrainChannelProtobuf::Channel4,
		}
	}
}

use defmt::Format;
use messages::argus::adc::AdcDevice as AdcDeviceProtobuf;
use serde::{Deserialize, Serialize};
use strum::EnumCount;

// Called AdcDevice to not clash with embassy::adc::Adc
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Format, Serialize, Deserialize, EnumCount)]
pub enum AdcDevice {
	AdcDevice1 = 0,
	AdcDevice2 = 1,
}

// Support for implicit conversion from usize to AdcDevice
impl From<usize> for AdcDevice {
	fn from(value: usize) -> Self {
		match value {
			0 => AdcDevice::AdcDevice1,
			1 => AdcDevice::AdcDevice2,
			_ => panic!("Invalid ADC index: {}", value),
		}
	}
}

impl AdcDevice {
	pub fn to_protobuf(&self) -> AdcDeviceProtobuf {
		match self {
			AdcDevice::AdcDevice1 => AdcDeviceProtobuf::AdcDevice1,
			AdcDevice::AdcDevice2 => AdcDeviceProtobuf::AdcDevice2,
		}
	}
}

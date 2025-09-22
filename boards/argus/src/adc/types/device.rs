use defmt::Format;
use serde::{Deserialize, Serialize};

// Called AdcDevice to not clash with embassy::adc::Adc
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Format, Serialize, Deserialize)]
pub enum AdcDevice {
	Adc1 = 0,
	Adc2 = 1,
}

// Support for implicit conversion from usize to AdcDevice
impl From<usize> for AdcDevice {
	fn from(value: usize) -> Self {
		match value {
			0 => AdcDevice::Adc1,
			1 => AdcDevice::Adc2,
			_ => panic!("Invalid ADC index: {}", value),
		}
	}
}

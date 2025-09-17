// General configurations for the Argus board
// Note: For configurations for specific modules, see their respective config.rs files i.e. sd/config.rs, temperature/config.rs, etc.

use defmt::Format;
use serde::Serialize;

// Number of ADC chips in the system
pub const ADC_COUNT: usize = 2;

// Called AdcDevice to not clash with embassy::adc::Adc
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Format, Serialize)]
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

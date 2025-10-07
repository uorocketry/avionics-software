use core::fmt::Debug;
use core::hash::Hash;
use core::str::FromStr;

use defmt::Format;
use num_traits::Float;
use serde::{Deserialize, Serialize};

use crate::adc::types::AdcDevice;
use crate::sd::csv::types::SerializeCSV;
use crate::sd::types::Line;

// Represents a linear transformation applied to a sensor reading
// corrected_value = value_with_error * gain + offset
#[derive(Debug, Clone, Copy, Format, Serialize, Deserialize)]
pub struct LinearTransformation<C, V> {
	pub adc: AdcDevice,
	pub channel: C,
	pub gain: V,
	pub offset: V,
}

impl<C, V> LinearTransformation<C, V>
where
	C: Default + Debug + Clone + Copy + Eq + PartialEq + Hash + Format + Serialize + for<'de> Deserialize<'de>,
	V: Float + Serialize + for<'de> Deserialize<'de>,
{
	pub fn apply(
		&self,
		raw_value: V,
	) -> V {
		raw_value * self.gain + self.offset
	}
}

impl<C, V> Default for LinearTransformation<C, V>
where
	C: Default + Debug + Clone + Copy + Eq + PartialEq + Hash + Format + Serialize + for<'de> Deserialize<'de>,
	V: Float + Serialize + for<'de> Deserialize<'de>,
{
	fn default() -> Self {
		Self {
			adc: AdcDevice::Adc1, // Default to ADC1
			channel: C::default(),
			gain: V::one(),    // Default to unity gain
			offset: V::zero(), // Default to zero offset
		}
	}
}

impl<C, V> SerializeCSV for LinearTransformation<C, V>
where
	C: Default + Debug + Clone + Copy + Eq + PartialEq + Hash + Format + Serialize + for<'de> Deserialize<'de>,
	V: Float + Serialize + for<'de> Deserialize<'de>,
{
	fn get_csv_header() -> Line {
		Line::from_str("ADC Index,Channel Index,Gain,Offset\n").unwrap()
	}
}

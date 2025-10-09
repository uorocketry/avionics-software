use core::fmt::Debug;
use core::hash::Hash;
use core::str::FromStr;

use csv::SerializeCSV;
use defmt::Format;
use num_traits::Float;
use serde::{Deserialize, Serialize};
use strum::EnumCount;

use crate::adc::types::AdcDevice;
use crate::sd::config::MAX_LINE_LENGTH;
use crate::sd::types::Line;

// Represents a linear transformation applied to a sensor reading
// corrected_value = value_with_error * gain + offset
#[derive(Debug, Clone, Copy, Format, Serialize, Deserialize)]
pub struct LinearTransformation<Channel, ChannelValue> {
	pub adc: AdcDevice,
	pub channel: Channel,
	pub gain: ChannelValue,
	pub offset: ChannelValue,
}

impl<Channel, ChannelValue> LinearTransformation<Channel, ChannelValue>
where
	Channel: EnumCount + Default + Debug + Clone + Copy + Eq + PartialEq + Hash + Format + Serialize + for<'de> Deserialize<'de>,
	ChannelValue: Float + Serialize + for<'de> Deserialize<'de>,
{
	pub fn apply(
		&self,
		raw_value: ChannelValue,
	) -> ChannelValue {
		raw_value * self.gain + self.offset
	}
}

impl<Channel, ChannelValue> Default for LinearTransformation<Channel, ChannelValue>
where
	Channel: EnumCount + Default + Debug + Clone + Copy + Eq + PartialEq + Hash + Format + Serialize + for<'de> Deserialize<'de>,
	ChannelValue: Float + Serialize + for<'de> Deserialize<'de>,
{
	fn default() -> Self {
		Self {
			adc: AdcDevice::Adc1, // Default to ADC1
			channel: Channel::default(),
			gain: ChannelValue::one(),    // Default to unity gain
			offset: ChannelValue::zero(), // Default to zero offset
		}
	}
}

impl<Channel, ChannelValue> SerializeCSV<MAX_LINE_LENGTH> for LinearTransformation<Channel, ChannelValue>
where
	Channel: EnumCount + Default + Debug + Clone + Copy + Eq + PartialEq + Hash + Format + Serialize + for<'de> Deserialize<'de>,
	ChannelValue: Float + Serialize + for<'de> Deserialize<'de>,
{
	fn get_csv_header() -> Line {
		Line::from_str("ADC Index,Channel Index,Gain,Offset\n").unwrap()
	}
}

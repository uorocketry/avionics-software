use core::fmt::Debug;
use core::hash::Hash;
use core::str::FromStr;

use defmt::Format;
use num_traits::Float;
use serde::{Deserialize, Serialize};
use strum::EnumCount;
use uor_utils::csv::SerializeCSV;

use crate::adc::types::AdcDevice;
use crate::sd::config::MAX_LINE_LENGTH;
use crate::sd::types::Line;

// Represents a linear transformation applied to a sensor reading
// corrected_value = value_with_error * scale + offset
#[derive(Debug, Clone, Copy, Format, Serialize, Deserialize)]
pub struct LinearTransformation<Channel, ChannelValue> {
	pub adc: AdcDevice,
	pub channel: Channel,
	pub scale: ChannelValue,
	pub offset: ChannelValue,
}

impl<Channel, ChannelValue> LinearTransformation<Channel, ChannelValue>
where
	Channel: ChannelMarker,
	ChannelValue: ChannelValueMarker,
{
	pub fn apply(
		&self,
		raw_value: ChannelValue,
	) -> ChannelValue {
		raw_value * self.scale + self.offset
	}
}

impl<Channel, ChannelValue> Default for LinearTransformation<Channel, ChannelValue>
where
	Channel: ChannelMarker,
	ChannelValue: ChannelValueMarker,
{
	fn default() -> Self {
		Self {
			adc: AdcDevice::AdcDevice1, // Default to ADC1
			channel: Channel::default(),
			scale: ChannelValue::one(),   // Default to unity gain
			offset: ChannelValue::zero(), // Default to zero offset
		}
	}
}

impl<Channel, ChannelValue> SerializeCSV<MAX_LINE_LENGTH> for LinearTransformation<Channel, ChannelValue>
where
	Channel: ChannelMarker,
	ChannelValue: ChannelValueMarker,
{
	fn get_csv_header() -> Line {
		Line::from_str("ADC Index,Channel Index,Scale,Offset").unwrap()
	}
}

pub trait ChannelMarker: EnumCount + Default + Debug + Clone + Copy + Eq + PartialEq + Hash + Format + Serialize + for<'de> Deserialize<'de> {}

impl<T> ChannelMarker for T where
	T: EnumCount + Default + Debug + Clone + Copy + Eq + PartialEq + Hash + Format + Serialize + for<'de> Deserialize<'de>
{
}

pub trait ChannelValueMarker: Float + Serialize + for<'de> Deserialize<'de> + Format {}

impl<T> ChannelValueMarker for T where T: Float + Serialize + for<'de> Deserialize<'de> + Format {}

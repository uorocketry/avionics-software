use core::str::FromStr;

use defmt::{error, info};
use heapless::LinearMap;
use uor_utils::csv::SerializeCSV;
use uor_utils::utils::types::AsyncMutex;

use crate::adc::types::AdcDevice;
use crate::linear_transformation::types::{ChannelMarker, ChannelValueMarker, LinearTransformation};
use crate::sd::service::SDCardService;
use crate::sd::types::{FileName, OperationScope, SdCardError};

// SHOULD DO: cleanup the trait bounds
pub struct LinearTransformationService<Channel, ChannelValue, const ADC_COUNT: usize, const CHANNEL_COUNT: usize>
where
	Channel: ChannelMarker,
	ChannelValue: ChannelValueMarker, {
	pub sd_card_service: &'static AsyncMutex<SDCardService>,
	pub file_name: &'static str,

	// Linear transformations that are applied on top of the raw readings for each ADC and channel
	pub transformations: LinearMap<AdcDevice, LinearMap<Channel, LinearTransformation<Channel, ChannelValue>, CHANNEL_COUNT>, ADC_COUNT>,
}

impl<Channel, ChannelValue, const ADC_COUNT: usize, const CHANNEL_COUNT: usize>
	LinearTransformationService<Channel, ChannelValue, ADC_COUNT, CHANNEL_COUNT>
where
	Channel: ChannelMarker,
	ChannelValue: ChannelValueMarker,
{
	pub fn new(
		sd_card_service: &'static AsyncMutex<SDCardService>,
		file_name: &'static str,
	) -> Self {
		Self {
			sd_card_service,
			file_name,
			transformations: LinearMap::default(),
		}
	}

	pub async fn load_transformations(&mut self) -> Result<(), SdCardError> {
		let result = self
			.sd_card_service
			.lock()
			.await
			.read(OperationScope::Root, FileName::from_str(self.file_name).unwrap(), |line| {
				if *line == LinearTransformation::<Channel, ChannelValue>::get_csv_header() {
					return true; // Skip header line
				}

				let result = LinearTransformation::<Channel, ChannelValue>::from_csv_line(line);
				match result {
					Ok(transformation) => {
						self.register_transformation(transformation);
						info!("Loaded linear transformation: {:?}", transformation);
					}
					Err(e) => {
						error!("Error parsing linear transformation for line '{}': {:?}", line.as_str(), e);
					}
				}
				true // Continue reading
			});

		match result {
			Ok(_) => (),
			Err(SdCardError::NotFound) => {
				// If transformations not found, keep using the defaults and ignore this error.
				info!("Linear transformations file not found, using defaults. Gain = 1, Offset = 0");
			}
			Err(e) => return Err(e),
		}
		Ok(())
	}

	pub fn register_transformation(
		&mut self,
		transformation: LinearTransformation<Channel, ChannelValue>,
	) {
		if !self.transformations.contains_key(&transformation.adc) {
			let _ = self.transformations.insert(transformation.adc, LinearMap::new());
		}
		let map = self.transformations.get_mut(&transformation.adc).unwrap();
		let _ = map.insert(transformation.channel, transformation);
	}

	pub fn apply_transformation(
		&self,
		adc: AdcDevice,
		channel: Channel,
		raw_value: ChannelValue,
	) -> ChannelValue {
		if let Some(channel_map) = self.transformations.get(&adc) {
			if let Some(transformation) = channel_map.get(&channel) {
				return transformation.apply(raw_value);
			}
		}
		raw_value // If no transformation found, return the raw value
	}

	pub fn deregister_transformation(
		&mut self,
		adc: AdcDevice,
		channel: Channel,
	) {
		if let Some(channel_map) = self.transformations.get_mut(&adc) {
			let _ = channel_map.remove(&channel);
		}
	}

	pub async fn save_transformation(
		&mut self,
		transformation: LinearTransformation<Channel, ChannelValue>,
	) -> Result<(), SdCardError> {
		info!("Saving linear transformation: {:?}", transformation);
		let mut sd_card_service = self.sd_card_service.lock().await;
		let path = FileName::from_str(self.file_name).unwrap();
		if !(sd_card_service.file_exists(OperationScope::Root, path.clone())?) {
			sd_card_service.write(
				OperationScope::Root,
				path.clone(),
				LinearTransformation::<Channel, ChannelValue>::get_csv_header(),
			)?;
		}

		sd_card_service.write(OperationScope::Root, path.clone(), transformation.to_csv_line())?;
		self.register_transformation(transformation);

		Ok(())
	}
}

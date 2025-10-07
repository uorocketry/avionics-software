use core::fmt::Debug;
use core::hash::Hash;
use core::str::FromStr;

use defmt::{info, Format};
use heapless::LinearMap;
use num_traits::Float;
use serde::{Deserialize, Serialize};

use crate::adc::config::ADC_COUNT;
use crate::adc::types::AdcDevice;
use crate::linear_transformation::types::LinearTransformation;
use crate::sd::csv::types::SerializeCSV;
use crate::sd::service::SDCardService;
use crate::sd::types::{FileName, OperationScope, SdCardError};
use crate::utils::types::AsyncMutex;

pub struct LinearTransformationService<C, V, const CHANNEL_COUNT: usize> {
	pub sd_card_service: &'static AsyncMutex<SDCardService>,
	pub file_name: &'static str,

	// Linear transformations that are applied on top of the raw readings for each ADC and channel
	pub transformations: LinearMap<AdcDevice, LinearMap<C, LinearTransformation<C, V>, CHANNEL_COUNT>, ADC_COUNT>,
}

impl<C, V, const CHANNEL_COUNT: usize> LinearTransformationService<C, V, CHANNEL_COUNT>
where
	C: Default + Debug + Clone + Copy + Eq + PartialEq + Hash + Format + Serialize + for<'de> Deserialize<'de>,
	V: Float + Serialize + for<'de> Deserialize<'de>,
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
				if *line == LinearTransformation::<C, V>::get_csv_header() {
					return true; // Skip header line
				}
				let transformation = LinearTransformation::<C, V>::from_csv_line(line);
				self.register_transformation(transformation);
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
		transformation: LinearTransformation<C, V>,
	) {
		if !self.transformations.contains_key(&transformation.adc) {
			let _ = self.transformations.insert(transformation.adc, LinearMap::new());
		}
		let map = self.transformations.get_mut(&transformation.adc).unwrap();
		let _ = map.insert(transformation.channel, transformation);
	}

	pub fn ensure_transformation_applied(
		&self,
		adc: AdcDevice,
		channel: C,
		raw_value: V,
	) -> V {
		if let Some(channel_map) = self.transformations.get(&adc) {
			if let Some(transformation) = channel_map.get(&channel) {
				return transformation.apply(raw_value);
			}
		}
		raw_value // If no transformation found, return the raw value
	}

	pub async fn save_transformation(
		&mut self,
		transformation: LinearTransformation<C, V>,
	) -> Result<(), SdCardError> {
		let mut sd_card_service = self.sd_card_service.lock().await;
		let path = FileName::from_str(self.file_name).unwrap();
		if !(sd_card_service.file_exists(OperationScope::Root, path.clone())?) {
			sd_card_service.write(OperationScope::Root, path.clone(), LinearTransformation::<C, V>::get_csv_header())?;
		}

		sd_card_service.write(OperationScope::Root, path.clone(), transformation.to_csv_line())?;
		self.register_transformation(transformation);

		Ok(())
	}
}

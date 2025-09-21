use core::str::FromStr;

use embassy_time::Instant;
use heapless::LinearMap;

use crate::adc::driver::types::{DataRate, Filter, Gain, ReferenceRange};
use crate::adc::service::AdcService;
use crate::config::{AdcDevice, ADC_COUNT};
use crate::sd::csv::types::SerializeCSV;
use crate::sd::service::SDCardService;
use crate::sd::types::{FileName, OperationScope};
use crate::serial::service::SerialService;
use crate::temperature::types::{
	LinearTransformation, TemperatureServiceError, ThermocoupleChannel, ThermocoupleReading, ThermocoupleReadingQueue, CHANNEL_COUNT,
	LINEAR_TRANSFORMATIONS_FILE_NAME,
};
use crate::utils::types::AsyncMutex;

// A channel for buffering the temperature readings and decoupling the logging to sd task from the measurement task
pub static THERMOCOUPLE_READING_QUEUE: ThermocoupleReadingQueue = ThermocoupleReadingQueue::new();

pub struct TemperatureService {
	// Other services are passed by a mutex to ensure safe concurrent access
	pub adc_service: &'static AsyncMutex<AdcService>,
	pub sd_card_service: &'static AsyncMutex<SDCardService>,
	pub serial_service: &'static AsyncMutex<SerialService>,

	// Linear transformations that are applied on top of the raw readings for each ADC and channel
	pub transformations: LinearMap<AdcDevice, LinearMap<ThermocoupleChannel, LinearTransformation, CHANNEL_COUNT>, ADC_COUNT>,
}

impl TemperatureService {
	pub fn new(
		adc_service: &'static AsyncMutex<AdcService>,
		sd_card_service: &'static AsyncMutex<SDCardService>,
		serial_service: &'static AsyncMutex<SerialService>,
	) -> Self {
		Self {
			adc_service,
			sd_card_service,
			serial_service,
			transformations: LinearMap::new(),
		}
	}

	pub async fn setup(&mut self) -> Result<(), TemperatureServiceError> {
		for driver in self.adc_service.lock().await.drivers.iter_mut() {
			driver.reference_range = ReferenceRange::Avdd;
			driver.data_rate = DataRate::Sps100;
			driver.filter = Filter::Sinc3;
			driver.enable_internal_reference = true;
			driver.gain = Gain::G32;
			driver.apply_configurations().await?;
		}

		self.load_transformations().await?;
		Ok(())
	}

	pub async fn read_thermocouple(
		&mut self,
		adc: AdcDevice,
		channel: ThermocoupleChannel,
	) -> Result<ThermocoupleReading, TemperatureServiceError> {
		let mut adc_service = self.adc_service.lock().await;

		// Get the respective "adc channel" pair for the "thermocouple channel"
		let (positive_channel, negative_channel) = channel.to_analog_input_channel_pair();

		let voltage = adc_service.drivers[adc as usize]
			.read_differential(positive_channel, negative_channel)
			.await?;

		let thermocouple_reading = ThermocoupleReading {
			timestamp_in_milliseconds: Instant::now().as_millis(),
			voltage_in_millivolts: voltage * 1000.0,
			uncompensated_temperature_in_celsius: None, // Placeholder for actual reading
			compensated_temperature_in_celsius: None,   // Placeholder for actual compensation logic
			cold_junction_temperature_in_celsius: None, // Placeholder for actual cold junction temperature
		};

		Ok(thermocouple_reading)
	}

	pub async fn load_transformations(&mut self) -> Result<(), TemperatureServiceError> {
		self.sd_card_service.lock().await.read(
			OperationScope::Root,
			FileName::from_str(LINEAR_TRANSFORMATIONS_FILE_NAME).unwrap(),
			|line| {
				if *line == LinearTransformation::get_csv_header() {
					return true; // Skip header line
				}
				let transformation = LinearTransformation::from_csv_line(line);
				self.load_transformation(transformation);
				return true; // Continue reading
			},
		)?;
		Ok(())
	}

	pub fn load_transformation(
		&mut self,
		transformation: LinearTransformation,
	) {
		if !self.transformations.contains_key(&transformation.adc) {
			let _ = self.transformations.insert(transformation.adc, LinearMap::new());
		}
		let map = self.transformations.get_mut(&transformation.adc).unwrap();
		let _ = map.insert(transformation.channel, transformation);
	}
}

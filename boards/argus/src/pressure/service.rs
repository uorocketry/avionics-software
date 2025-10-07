use embassy_time::Instant;

use crate::adc::driver::types::{DataRate, Filter, Gain, ReferenceRange};
use crate::adc::service::AdcService;
use crate::adc::types::AdcDevice;
use crate::linear_transformation::service::LinearTransformationService;
use crate::pressure::config::{LINEAR_TRANSFORMATIONS_FILE_NAME, PRESSURE_CHANNEL_COUNT};
use crate::pressure::types::{PressureChannel, PressureReading, PressureReadingQueue, PressureServiceError};
use crate::sd::service::SDCardService;
use crate::serial::service::SerialService;
use crate::utils::types::AsyncMutex;

// A channel for buffering the pressure readings and decoupling the logging to sd task from the measurement task
pub static PRESSURE_READING_QUEUE: PressureReadingQueue = PressureReadingQueue::new();

pub struct PressureService {
	// Other services are passed by a mutex to ensure safe concurrent access
	pub adc_service: &'static AsyncMutex<AdcService>,
	pub sd_card_service: &'static AsyncMutex<SDCardService>,
	pub serial_service: &'static AsyncMutex<SerialService>,

	// Linear transformation service to apply error corrections obtained from calibration to raw readings
	pub linear_transformation_service: LinearTransformationService<PressureChannel, f64, PRESSURE_CHANNEL_COUNT>,
}

impl PressureService {
	pub fn new(
		adc_service: &'static AsyncMutex<AdcService>,
		sd_card_service: &'static AsyncMutex<SDCardService>,
		serial_service: &'static AsyncMutex<SerialService>,
	) -> Self {
		Self {
			adc_service,
			sd_card_service,
			serial_service,
			linear_transformation_service: LinearTransformationService::new(sd_card_service, LINEAR_TRANSFORMATIONS_FILE_NAME),
		}
	}

	pub async fn setup(&mut self) -> Result<(), PressureServiceError> {
		for driver in self.adc_service.lock().await.drivers.iter_mut() {
			driver.reference_range = ReferenceRange::Avdd;
			driver.data_rate = DataRate::Sps100;
			driver.filter = Filter::Sinc3;
			driver.enable_internal_reference = true;
			driver.gain = Gain::G32;
			driver.delay_after_setting_channel = 50; // 50 ms delay to allow the ADC to stabilize after switching channels
			driver.apply_configurations().await?;
		}

		self.linear_transformation_service.load_transformations().await?;
		Ok(())
	}

	pub async fn read_pressure_sensor(
		&mut self,
		adc: AdcDevice,
		channel: PressureChannel,
	) -> Result<PressureReading, PressureServiceError> {
		let mut adc_service = self.adc_service.lock().await;

		// Get the respective "adc channel" pair for the "thermocouple channel"
		let (positive_channel, negative_channel) = channel.to_analog_input_channel_pair();

		// Read the voltage from the ADC in millivolts
		let voltage = adc_service.drivers[adc as usize]
			.read_differential(positive_channel, negative_channel)
			.await? * 1000.0; // Convert to millivolts

		let thermocouple_reading = PressureReading {
			timestamp: Instant::now().as_millis(),
			voltage,
			pressure: 0.0,    // Placeholder, actual pressure calculation can be added later
			temperature: 0.0, // Placeholder, actual temperature calculation can be added
		};

		Ok(thermocouple_reading)
	}
}

use defmt::{error, info};
use embassy_time::{Instant, Timer};
use serial::service::SerialService;
use strum::EnumCount;

use crate::adc::driver::types::{DataRate, Filter, Gain, ReferenceRange};
use crate::adc::service::AdcService;
use crate::adc::types::AdcDevice;
use crate::linear_transformation::service::LinearTransformationService;
use crate::pressure::config::LINEAR_TRANSFORMATIONS_FILE_NAME;
use crate::pressure::types::{PressureChannel, PressureReading, PressureReadingQueue, PressureServiceError};
use crate::sd::service::SDCardService;
use crate::session::service::SessionService;
use crate::utils::types::AsyncMutex;

// A channel for buffering the pressure readings and decoupling the logging to sd task from the measurement task
pub static PRESSURE_READING_QUEUE: PressureReadingQueue = PressureReadingQueue::new();

pub struct PressureService<const ADC_COUNT: usize> {
	// Other services are passed by a mutex to ensure safe concurrent access
	pub adc_service: &'static AsyncMutex<AdcService<ADC_COUNT>>,
	pub sd_card_service: &'static AsyncMutex<SDCardService>,
	pub serial_service: &'static AsyncMutex<SerialService>,
	pub session_service: &'static AsyncMutex<SessionService>,

	// Linear transformations that are applied on top of the raw readings for each ADC and channel
	pub linear_transformation_service: LinearTransformationService<PressureChannel, f64, ADC_COUNT, { PressureChannel::COUNT }>,
}

impl<const ADC_COUNT: usize> PressureService<ADC_COUNT> {
	pub fn new(
		adc_service: &'static AsyncMutex<AdcService<ADC_COUNT>>,
		sd_card_service: &'static AsyncMutex<SDCardService>,
		serial_service: &'static AsyncMutex<SerialService>,
		session_service: &'static AsyncMutex<SessionService>,
	) -> Self {
		Self {
			adc_service,
			sd_card_service,
			serial_service,
			session_service,
			linear_transformation_service: LinearTransformationService::new(sd_card_service, LINEAR_TRANSFORMATIONS_FILE_NAME),
		}
	}

	pub async fn setup(&mut self) -> Result<(), PressureServiceError> {
		// Delay for 100ms to ensure ADCs are powered up
		Timer::after_millis(100).await;

		for driver in self.adc_service.lock().await.drivers.iter_mut() {
			driver.reference_range = ReferenceRange::Avdd;
			driver.data_rate = DataRate::Sps100;
			driver.filter = Filter::Sinc3;
			driver.enable_internal_reference = true;
			driver.gain = Gain::G32;
			driver.delay_after_setting_channel = 50; // 50 ms delay to allow the ADC to stabilize after switching channels
			driver.apply_configurations().await?;
		}

		match self.linear_transformation_service.load_transformations().await {
			Err(e) => error!("Failed to load linear transformations: {:?}", e),
			_ => {}
		}
		Ok(())
	}

	pub async fn read_pressure(
		&mut self,
		adc: AdcDevice,
		channel: PressureChannel,
	) -> Result<PressureReading, PressureServiceError> {
		let mut adc_service = self.adc_service.lock().await;

		// Get the respective "adc channel" pair for the "pressure channel"
		let (positive_channel, negative_channel) = channel.to_analog_input_channel_pair();

		// Read the voltage from the ADC in millivolts
		let voltage = adc_service.drivers[adc as usize]
			.read_differential(positive_channel, negative_channel)
			.await? * 1000.0; // Convert to millivolts

		// Apply any linear transformations to get the pressure in psi
		let pressure = self.linear_transformation_service.apply_transformation(adc, channel, voltage as f64);

		let pressure_reading = PressureReading {
			local_session: self.session_service.lock().await.current_session.clone(),
			adc_device: adc,
			pressure_channel: channel,
			recorded_at: Instant::now().as_millis(),
			voltage,
			pressure,
			temperature: 0.0, // SHOULD DO: replace once NTC temperature measurement is implemented
		};

		Ok(pressure_reading)
	}
}

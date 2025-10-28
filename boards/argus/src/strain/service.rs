use defmt::error;
use embassy_time::Instant;
use strum::EnumCount;

use crate::adc::driver::types::{DataRate, Filter, Gain, ReferenceRange};
use crate::adc::service::AdcService;
use crate::adc::types::AdcDevice;
use crate::linear_transformation::service::LinearTransformationService;
use crate::sd::service::SDCardService;
use crate::serial::service::SerialService;
use crate::session::service::SessionService;
use crate::strain::config::LINEAR_TRANSFORMATIONS_FILE_NAME;
use crate::strain::types::{StrainChannel, StrainReading, StrainReadingQueue, StrainServiceError};
use crate::utils::types::AsyncMutex;

// A channel for buffering the strain readings and decoupling the logging to sd task from the measurement task
pub static STRAIN_READING_QUEUE: StrainReadingQueue = StrainReadingQueue::new();

pub struct StrainService<const ADC_COUNT: usize> {
	// Other services are passed by a mutex to ensure safe concurrent access
	pub adc_service: &'static AsyncMutex<AdcService<ADC_COUNT>>,
	pub sd_card_service: &'static AsyncMutex<SDCardService>,
	pub serial_service: &'static AsyncMutex<SerialService>,
	pub session_service: &'static AsyncMutex<SessionService>,

	// Linear transformations that are applied on top of the raw readings for each ADC and channel
	pub linear_transformation_service: LinearTransformationService<StrainChannel, f64, ADC_COUNT, { StrainChannel::COUNT }>,
}

impl<const ADC_COUNT: usize> StrainService<ADC_COUNT> {
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

	pub async fn setup(&mut self) -> Result<(), StrainServiceError> {
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

	pub async fn read_strain(
		&mut self,
		adc: AdcDevice,
		channel: StrainChannel,
	) -> Result<StrainReading, StrainServiceError> {
		let mut adc_service = self.adc_service.lock().await;

		// Get the respective "adc channel" pair for the "strain channel"
		let (positive_channel, negative_channel) = channel.to_analog_input_channel_pair();

		// Read the voltage from the ADC in millivolts
		let voltage = adc_service.drivers[adc as usize]
			.read_differential(positive_channel, negative_channel)
			.await? * 1000.0; // Convert to millivolts

		// Apply any linear transformations to get the strain in psi
		let strain = self.linear_transformation_service.apply_transformation(adc, channel, voltage as f64);

		let strain_reading = StrainReading {
			local_session: self.session_service.lock().await.current_session.clone(),
			adc_device: adc,
			strain_channel: channel,
			recorded_at: Instant::now().as_millis(),
			voltage,
			strain,
		};

		Ok(strain_reading)
	}
}

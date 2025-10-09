use embassy_time::Instant;
use strum::EnumCount;

use crate::adc::driver::types::{AnalogChannel, DataRate, Filter, Gain, ReferenceRange};
use crate::adc::service::AdcService;
use crate::adc::types::AdcDevice;
use crate::linear_transformation::service::LinearTransformationService;
use crate::sd::service::SDCardService;
use crate::serial::service::SerialService;
use crate::temperature::config::{LINEAR_TRANSFORMATIONS_FILE_NAME, RTD_RESISTANCE_AT_0C};
use crate::temperature::rtd;
use crate::temperature::thermocouple::type_k;
use crate::temperature::types::{TemperatureServiceError, ThermocoupleChannel, ThermocoupleReading, ThermocoupleReadingQueue};
use crate::utils::types::AsyncMutex;

// A channel for buffering the temperature readings and decoupling the logging to sd task from the measurement task
pub static THERMOCOUPLE_READING_QUEUE: ThermocoupleReadingQueue = ThermocoupleReadingQueue::new();

pub struct TemperatureService<const ADC_COUNT: usize> {
	// Other services are passed by a mutex to ensure safe concurrent access
	pub adc_service: &'static AsyncMutex<AdcService<ADC_COUNT>>,
	pub sd_card_service: &'static AsyncMutex<SDCardService>,
	pub serial_service: &'static AsyncMutex<SerialService>,

	// Store the last RTD reading in Celsius to use for cold junction compensation
	// This is cached here to avoid reading the RTD multiple times when reading multiple thermocouples
	// We have one RTD per ADC, so we store an array of last readings
	pub last_rtd_reading: [Option<f32>; ADC_COUNT],

	// Linear transformations that are applied on top of the raw readings for each ADC and channel
	pub linear_transformation_service: LinearTransformationService<ThermocoupleChannel, f64, ADC_COUNT, { ThermocoupleChannel::COUNT }>,
}

impl<const ADC_COUNT: usize> TemperatureService<ADC_COUNT> {
	pub fn new(
		adc_service: &'static AsyncMutex<AdcService<ADC_COUNT>>,
		sd_card_service: &'static AsyncMutex<SDCardService>,
		serial_service: &'static AsyncMutex<SerialService>,
	) -> Self {
		Self {
			adc_service,
			sd_card_service,
			serial_service,
			last_rtd_reading: [None; ADC_COUNT],
			linear_transformation_service: LinearTransformationService::new(sd_card_service, LINEAR_TRANSFORMATIONS_FILE_NAME),
		}
	}

	pub async fn setup(&mut self) -> Result<(), TemperatureServiceError> {
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

	pub async fn read_thermocouple(
		&mut self,
		adc: AdcDevice,
		channel: ThermocoupleChannel,
	) -> Result<ThermocoupleReading, TemperatureServiceError> {
		let mut adc_service = self.adc_service.lock().await;

		// Get the respective "adc channel" pair for the "thermocouple channel"
		let (positive_channel, negative_channel) = channel.to_analog_input_channel_pair();

		// Read the voltage from the ADC in millivolts
		let voltage = adc_service.drivers[adc as usize]
			.read_differential(positive_channel, negative_channel)
			.await? * 1000.0; // Convert to millivolts

		// Get the cold junction temperature from the last RTD reading for this ADC
		let cold_junction_temperature = self.last_rtd_reading[adc as usize].unwrap_or(0.0);

		let thermocouple_reading = ThermocoupleReading {
			timestamp: Instant::now().as_millis(),
			voltage,
			uncompensated_temperature: type_k::convert_voltage_to_temperature(voltage as f64)?,
			compensated_temperature: type_k::convert_voltage_to_temperature_with_cold_junction_compensation(
				voltage as f64,
				cold_junction_temperature as f64,
			)?,
			cold_junction_temperature,
		};

		Ok(thermocouple_reading)
	}

	pub async fn read_rtd(
		&mut self,
		adc: AdcDevice,
	) -> Result<f32, TemperatureServiceError> {
		let mut adc_service = self.adc_service.lock().await;
		let driver = &mut adc_service.drivers[adc as usize];
		let previous_gain = driver.gain;

		// Set the gain to 1 for RTD measurement to avoid saturating the ADC
		driver.gain = Gain::G1;
		driver.apply_gain_and_data_rate_configuration().await?;
		driver.wait_for_next_data().await;

		// Perform the measurement at the gain of 1

		// Note: This is based on Argus V2 design as of September 22, 2025
		// The AIN8-9 sequence is flipped accidentally so AIN9 is before the RTD and AIN8 is after the RTD
		let voltage_before_rtd = driver.read_single_ended(AnalogChannel::AIN9).await?;
		let voltage_after_rtd = driver.read_single_ended(AnalogChannel::AIN8).await?;

		// Restore the previous gain
		driver.gain = previous_gain;
		driver.apply_gain_and_data_rate_configuration().await?;

		// I = voltage_after_rtd / R6
		// measured_resistance = V_RTD / I = R6 * (voltage_before_rtd - voltage_after_rtd) / voltage_after_rtd
		const R6: f32 = 1000.0;
		let measured_resistance = R6 * (voltage_before_rtd - voltage_after_rtd) / voltage_after_rtd;
		let estimated_temperature = rtd::convert_resistance_to_temperature(RTD_RESISTANCE_AT_0C, measured_resistance);

		Ok(estimated_temperature)
	}
}

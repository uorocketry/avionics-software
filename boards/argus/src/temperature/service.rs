use embassy_time::Instant;
use heapless::String;

use crate::adc::driver::types::{DataRate, Filter, Gain, ReferenceRange};
use crate::adc::service::{AdcError, AdcService};
use crate::config::{AdcDevice, ADC_COUNT};
use crate::sd::service::SDCardService;
use crate::serial::service::{AsyncUartError, SerialService};
use crate::temperature::config::{ThermocoupleChannel, CHANNEL_COUNT};
use crate::temperature::types::{LinearTransformation, ThermocoupleReading};
use crate::utils::types::AsyncMutex;

pub struct TemperatureService {
	// Other services are passed by a mutex to ensure safe concurrent access
	pub adc_service: &'static AsyncMutex<AdcService>,
	pub sd_service: &'static AsyncMutex<SDCardService>,
	pub serial_service: &'static AsyncMutex<SerialService>,

	// Transformations that gets degrees in celsius from voltage readings for each channel of each ADC
	// transformations[adc_index][channel_index]
	pub transformations: [[LinearTransformation; CHANNEL_COUNT]; ADC_COUNT],
}

impl TemperatureService {
	pub fn new(
		adc_service: &'static AsyncMutex<AdcService>,
		sd_service: &'static AsyncMutex<SDCardService>,
		serial_service: &'static AsyncMutex<SerialService>,
	) -> Self {
		Self {
			adc_service,
			sd_service,
			serial_service,
			transformations: [[LinearTransformation::default(); CHANNEL_COUNT]; ADC_COUNT],
		}
	}

	pub async fn configure_adcs(&mut self) -> Result<(), AdcError> {
		for driver in self.adc_service.lock().await.drivers.iter_mut() {
			driver.reference_range = ReferenceRange::Avdd;
			driver.data_rate = DataRate::Sps100;
			driver.filter = Filter::Sinc3;
			driver.enable_internal_reference = true;
			driver.gain = Gain::G32;
			driver.apply_configurations().await?;
		}
		Ok(())
	}

	pub async fn read_thermocouple(
		&mut self,
		adc: AdcDevice,
		channel: ThermocoupleChannel,
	) -> Result<ThermocoupleReading, AdcError> {
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

	pub async fn calibrate(&mut self) -> Result<(), AsyncUartError> {
		let mut serial_service = self.serial_service.lock().await;
		let mut input = String::<256>::new();

		// Prompt for ADC index
		serial_service
			.write_str(
				"Starting temperature calibration. \
				Enter ADC index (Starts from 0):\"\n",
			)
			.await?;
		input.clear();
		serial_service.read_line(&mut input).await?;
		let adc_index: usize = input.trim().parse().unwrap_or(0);
		if adc_index >= ADC_COUNT {
			serial_service.write_str("Invalid ADC index.\n").await?;
			return Ok(());
		}

		// Prompt for channel
		serial_service.write_str("Enter channel index (Starts from 0):\n").await?;
		input.clear();
		serial_service.read_line(&mut input).await?;
		let channel_index: usize = input.trim().parse().unwrap_or(0);
		let channel = ThermocoupleChannel::from(channel_index);

		// SHOULD DO: finish this
		Ok(())
	}
}

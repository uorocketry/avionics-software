use embassy_time::Instant;

use crate::adc::driver::types::{DataRate, Filter, Gain, ReferenceRange};
use crate::adc::service::{AdcError, AdcService};
use crate::temperature::conversion::type_k::emf_to_temperature;
use crate::temperature::types::{AdcIndex, ThermocoupleChannel, ThermocoupleReading};

/// Temperature traits for ADC service. Enabled by the "temperature" feature.
pub trait TemperatureAdcService {
	async fn configure(&mut self) -> Result<(), AdcError>;
	async fn read_thermocouple(
		&mut self,
		adc_index: AdcIndex,
		channel: ThermocoupleChannel,
	) -> Result<ThermocoupleReading, AdcError>;
}

impl<const ADC_COUNT: usize> TemperatureAdcService for AdcService<ADC_COUNT> {
	async fn configure(&mut self) -> Result<(), AdcError> {
		for driver in self.drivers.iter_mut() {
			driver.reference_range = ReferenceRange::Avdd;
			driver.data_rate = DataRate::Sps100;
			driver.filter = Filter::Sinc3;
			driver.enable_internal_reference = true;
			driver.gain = Gain::G32;
			driver.apply_configurations().await?;
		}
		Ok(())
	}

	async fn read_thermocouple(
		&mut self,
		adc_index: AdcIndex,
		channel: ThermocoupleChannel,
	) -> Result<ThermocoupleReading, AdcError> {
		let (positive_channel, negative_channel) = channel.to_analog_input_channel_pair();
		let voltage = self.drivers[adc_index].read_differential(positive_channel, negative_channel).await?;
		let voltage_in_millivolts = voltage * 1000.0; // Convert to millivolts
		let thermocouple_reading = ThermocoupleReading {
			timestamp_in_milliseconds: Instant::now().as_millis(),
			voltage_in_millivolts,
			uncompensated_temperature_in_celsius: emf_to_temperature(voltage_in_millivolts), // Placeholder for actual reading
			compensated_temperature_in_celsius: None,                                        // Placeholder for actual compensation logic
			cold_junction_temperature_in_celsius: None,                                      // Placeholder for actual cold junction temperature
		};

		Ok(thermocouple_reading)
	}
}

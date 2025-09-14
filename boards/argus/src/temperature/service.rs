use crate::adc::driver::types::{ChannelShift, DataRate, Gain};
use crate::adc::service::{AdcService, AdcError};
use crate::temperature::types::{Thermocouple, ThermocoupleReading};

/// Temperature traits for ADC service. Enabled by the "temperature" feature.
pub trait TemperatureAdcService {
	async fn configure_for_temperature_measurement(&mut self) -> Result<(), AdcError>;
	async fn read_thermocouple(&mut self, adc_index: usize, channel_index: usize) -> Result<ThermocoupleReading, AdcError>;
}

impl<const ADC_COUNT: usize> TemperatureAdcService for AdcService<ADC_COUNT> {
	async fn configure_for_temperature_measurement(&mut self) -> Result<(), AdcError> {
		for driver in self.drivers.iter_mut() {
			driver.channel_shift = ChannelShift::MidSupply;
			driver.data_rate = DataRate::Sps4800;
			driver.enable_internal_reference_voltage = true;
			driver.gain = Gain::G32;
			driver.apply_configurations().await?;
		}
		Ok(())
	}

	async fn read_thermocouple(&mut self, adc_index: usize, channel_index: usize) -> Result<ThermocoupleReading, AdcError> {
		let (positive_channel, negative_channel) = Thermocouple::from(channel_index).to_analog_input_channel_pair();
		let voltage = self.drivers[adc_index].read_differential(positive_channel, negative_channel).await?;
		let thermocouple_reading = ThermocoupleReading {
			timestamp: 0, // Placeholder for actual timestamp logic
			voltage,
			compensated_temperature: 0.0, // Placeholder for actual compensation logic
			uncompensated_temperature: 0.0, // Placeholder for actual reading
			cold_junction_temperature: 0.0, // Placeholder for actual cold junction temperature
		};

		Ok(thermocouple_reading)
	}
}

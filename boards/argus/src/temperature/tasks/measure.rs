use defmt::debug;
use embassy_executor::task;

use crate::config::{AdcDevice, ADC_COUNT};
use crate::state_machine::service::StateMachineWorker;
use crate::state_machine::types::States;
use crate::temperature::service::{TemperatureService, THERMOCOUPLE_READING_QUEUE};
use crate::temperature::types::{ThermocoupleChannel, CHANNEL_COUNT};
use crate::utils::types::AsyncMutex;

// Task that iterates through the ADCs and channels, measures the temperature, and enqueues the readings to a channel
#[task]
pub async fn measure(
	mut worker: StateMachineWorker,
	temperature_service_mutex: &'static AsyncMutex<TemperatureService>,
) {
	// Configure the ADCs for temperature measurement
	temperature_service_mutex.lock().await.setup().await.unwrap();

	worker
		.run_while(States::Recording, async |_| -> Result<(), ()> {
			let mut temperature_service = temperature_service_mutex.lock().await;

			for adc_index in 0..ADC_COUNT {
				for channel_index in 0..CHANNEL_COUNT {
					let adc = AdcDevice::from(adc_index);
					let channel = ThermocoupleChannel::from(channel_index);
					let data = temperature_service.read_thermocouple(adc, channel).await;
					match data {
						Ok(data) => {
							debug!("ADC {} Channel {}: {}", adc, channel, data);
							THERMOCOUPLE_READING_QUEUE.send((adc, channel, data)).await;
						}
						Err(error) => {
							debug!("Error reading ADC {} Channel {}: {:?}", adc, channel, error);
							continue;
						}
					}
				}
			}
			Ok(())
		})
		.await
		.unwrap();
}

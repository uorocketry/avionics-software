use defmt::{debug, error};
use embassy_executor::task;
use strum::EnumCount;

use crate::adc::types::AdcDevice;
use crate::state_machine::service::StateMachineWorker;
use crate::state_machine::types::States;
use crate::temperature::service::{TemperatureService, THERMOCOUPLE_READING_QUEUE};
use crate::temperature::types::ThermocoupleChannel;
use crate::utils::types::AsyncMutex;

// Task that iterates through the ADCs and channels, measures the temperature, and enqueues the readings to a channel
#[task]
pub async fn measure_thermocouples(
	mut worker: StateMachineWorker,
	temperature_service_mutex: &'static AsyncMutex<TemperatureService<{ AdcDevice::COUNT }>>,
) {
	worker
		.run_while(States::Recording, async |_| -> Result<(), ()> {
			let mut temperature_service = temperature_service_mutex.lock().await;

			for adc_index in 0..AdcDevice::COUNT {
				for channel_index in 0..ThermocoupleChannel::COUNT {
					let adc = AdcDevice::from(adc_index);
					let channel = ThermocoupleChannel::from(channel_index);
					let data = temperature_service.read_thermocouple(adc, channel).await;
					match data {
						Ok(data) => {
							debug!("ADC {} Channel {}: {}", adc, channel, data);
							THERMOCOUPLE_READING_QUEUE.send((adc, channel, data)).await;
						}
						Err(err) => {
							error!("Error reading ADC {} Channel {}: {:?}", adc, channel, err);
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

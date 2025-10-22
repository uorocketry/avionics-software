use defmt::{debug, error};
use embassy_executor::task;
use strum::EnumCount;

use crate::adc::types::AdcDevice;
use crate::pressure::service::{PressureService, PRESSURE_READING_QUEUE};
use crate::pressure::types::PressureChannel;
use crate::state_machine::service::StateMachineWorker;
use crate::state_machine::types::States;
use crate::utils::types::AsyncMutex;

// Task that iterates through the ADCs and channels, measures the pressure, and enqueues the readings to a channel
#[task]
pub async fn measure_pressure(
	mut worker: StateMachineWorker,
	pressure_service_mutex: &'static AsyncMutex<PressureService<{ AdcDevice::COUNT }>>,
) {
	worker
		.run_while(&[States::Recording], async |_| -> Result<(), ()> {
			let mut pressure_service = pressure_service_mutex.lock().await;

			for adc_index in 0..AdcDevice::COUNT {
				for channel_index in 0..PressureChannel::COUNT {
					let adc = AdcDevice::from(adc_index);
					let channel = PressureChannel::from(channel_index);
					let data = pressure_service.read_pressure_sensor(adc, channel).await;
					match data {
						Ok(data) => {
							debug!("ADC {} Channel {}: {}", adc, channel, data);
							PRESSURE_READING_QUEUE.send((adc, channel, data)).await;
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

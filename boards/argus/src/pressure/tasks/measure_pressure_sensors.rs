use defmt::{error, info};
use embassy_executor::task;
use embassy_futures::yield_now;
use strum::EnumCount;
use utils::types::AsyncMutex;

use crate::adc::types::AdcDevice;
use crate::led_indicator::service::LedIndicatorService;
use crate::pressure::service::{PressureService, PRESSURE_READING_QUEUE};
use crate::pressure::types::PressureChannel;
use crate::state_machine::service::StateMachineWorker;
use crate::state_machine::types::States;

// Task that iterates through the ADCs and channels, measures the pressure, and enqueues the readings to a channel
#[task]
pub async fn measure_pressure_sensors(
	mut worker: StateMachineWorker,
	pressure_service_mutex: &'static AsyncMutex<PressureService<{ AdcDevice::COUNT }>>,
	led_indicator_service_mutex: &'static AsyncMutex<LedIndicatorService<2>>,
) {
	worker
		.run_while(&[States::Recording], async |_| -> Result<(), ()> {
			for adc_index in 0..AdcDevice::COUNT {
				for channel_index in 0..PressureChannel::COUNT {
					let adc = AdcDevice::from(adc_index);
					let channel = PressureChannel::from(channel_index);
					let data = pressure_service_mutex.lock().await.read_pressure(adc, channel).await;
					match data {
						Ok(pressure_reading) => {
							info!("{}", pressure_reading);
							PRESSURE_READING_QUEUE.send(pressure_reading).await;
						}
						Err(err) => {
							error!("Error reading ADC {} Channel {}: {:?}", adc, channel, err);
							continue;
						}
					}
				}
			}

			// Blink LED to indicate measurement cycle complete
			led_indicator_service_mutex.lock().await.blink(1).await;

			// Yield to allow other tasks to run, especially the NTC measurement task
			yield_now().await;
			Ok(())
		})
		.await
		.unwrap();
}

use defmt::{error, info};
use embassy_executor::task;
use embassy_futures::yield_now;
use strum::EnumCount;
use uor_utils::utils::types::AsyncMutex;

use crate::adc::types::AdcDevice;
use crate::led_indicator::service::LedIndicatorService;
use crate::state_machine::service::StateMachineWorker;
use crate::state_machine::types::States;
use crate::strain::service::{StrainService, STRAIN_READING_QUEUE};
use crate::strain::types::StrainChannel;

// Task that iterates through the ADCs and channels, measures the strain, and enqueues the readings to a channel
#[task]
pub async fn measure_strain(
	mut worker: StateMachineWorker,
	strain_service_mutex: &'static AsyncMutex<StrainService<{ AdcDevice::COUNT }>>,
	led_indicator_service_mutex: &'static AsyncMutex<LedIndicatorService<2>>,
) {
	worker
		.run_while(&[States::Recording], async |_| -> Result<(), ()> {
			for adc_index in 0..AdcDevice::COUNT {
				for channel_index in 0..StrainChannel::COUNT {
					let adc = AdcDevice::from(adc_index);
					let channel = StrainChannel::from(channel_index);
					let data = strain_service_mutex.lock().await.read_strain(adc, channel).await;
					match data {
						Ok(strain_reading) => {
							info!("{}", strain_reading);
							STRAIN_READING_QUEUE.send(strain_reading).await;
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

use defmt::{error, info};
use embassy_executor::task;
use embassy_futures::yield_now;
use strum::EnumCount;

use crate::adc::types::AdcDevice;
use crate::led_indicator::service::LedIndicatorService;
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
	led_indicator_service_mutex: &'static AsyncMutex<LedIndicatorService<2>>,
) {
	worker
		.run_while(&[States::Recording], async |_| -> Result<(), ()> {
			for adc_index in 0..AdcDevice::COUNT {
				for channel_index in 0..ThermocoupleChannel::COUNT {
					let adc = AdcDevice::from(adc_index);
					let channel = ThermocoupleChannel::from(channel_index);
					let data = temperature_service_mutex.lock().await.read_thermocouple(adc, channel).await;
					match data {
						Ok(thermocouple_reading) => {
							info!("{}", thermocouple_reading);
							THERMOCOUPLE_READING_QUEUE.send(thermocouple_reading).await;
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

			// Yield to allow other tasks to run, especially the RTD measurement task
			yield_now().await;
			Ok(())
		})
		.await
		.unwrap();
}

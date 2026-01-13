use defmt::error;
use embassy_executor::task;
use embassy_time::Timer;
use strum::EnumCount;
use utils::types::AsyncMutex;

use crate::adc::types::AdcDevice;
use crate::state_machine::service::StateMachineWorker;
use crate::state_machine::types::States;
use crate::temperature::config::RTD_MEASUREMENT_INTERVAL;
use crate::temperature::service::TemperatureService;

// Task that iterates through the ADCs and measures the RTDs at a slower interval than the thermocouples being read
// We don't need that frequent readings for the RTDs
#[task]
pub async fn measure_rtds(
	mut worker: StateMachineWorker,
	temperature_service_mutex: &'static AsyncMutex<TemperatureService<{ AdcDevice::COUNT }>>,
) {
	worker
		.run_while(&[States::Recording, States::Calibrating], async |_| -> Result<(), ()> {
			for adc_index in 0..AdcDevice::COUNT {
				let adc = AdcDevice::from(adc_index);
				match temperature_service_mutex.lock().await.refresh_rtd_reading(adc).await {
					Err(e) => {
						error!("Failed to read RTD on {:?}: {:?}", adc, e);
					}
					_ => {}
				}
			}

			// Delay the RTD measurement because it's not as critical as the thermocouples. We just need to read every once in a while
			Timer::after_millis(RTD_MEASUREMENT_INTERVAL).await;

			Ok(())
		})
		.await
		.unwrap();
}

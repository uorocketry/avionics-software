use defmt::{debug, error};
use embassy_executor::task;
use embassy_time::Timer;
use strum::EnumCount;

use crate::adc::types::AdcDevice;
use crate::state_machine::service::StateMachineWorker;
use crate::state_machine::types::States;
use crate::temperature::config::RTD_MEASUREMENT_INTERVAL;
use crate::temperature::service::TemperatureService;
use crate::utils::types::AsyncMutex;

// Task that iterates through the ADCs and measures the RTDs at a slower interval than the thermocouples being read
// We don't need that frequent readings for the RTDs
#[task]
pub async fn measure_rtds(
	mut worker: StateMachineWorker,
	temperature_service_mutex: &'static AsyncMutex<TemperatureService<{ AdcDevice::COUNT }>>,
) {
	worker
		.run_while(States::Recording, async |_| -> Result<(), ()> {
			for adc_index in 0..AdcDevice::COUNT {
				let mut temperature_service = temperature_service_mutex.lock().await;
				let adc = AdcDevice::from(adc_index);
				let result = temperature_service.read_rtd(adc).await;
				match result {
					Ok(data) => {
						debug!("RTD Temperature {}: {}C", adc, data);
						temperature_service.last_rtd_reading[adc_index] = Some(data);
					}
					Err(err) => {
						error!("Error reading RTD for {}: {:?}", adc, err);
					}
				}
			}

			// Delay the RTD measurement because it's not as critical as the thermocouples. We just need to read every once in a while
			Timer::after_millis(RTD_MEASUREMENT_INTERVAL).await;

			Ok(())
		})
		.await
		.unwrap();
}

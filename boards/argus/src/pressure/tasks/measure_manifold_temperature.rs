use embassy_executor::task;
use embassy_time::Timer;
use strum::EnumCount;
use utils::types::AsyncMutex;

use crate::adc::types::AdcDevice;
use crate::pressure::config::NTC_MEASUREMENT_INTERVAL;
use crate::pressure::service::PressureService;
use crate::state_machine::service::StateMachineWorker;
use crate::state_machine::types::States;

// Task that iterates through the ADCs and measures the NTCs at a slower interval than the pressures being read
// We don't need that frequent readings for the NTCs
#[task]
pub async fn measure_manifold_temperature(
	mut worker: StateMachineWorker,
	_pressure_service_mutex: &'static AsyncMutex<PressureService<{ AdcDevice::COUNT }>>,
) {
	worker
		.run_while(&[States::Recording, States::Calibrating], async |_| -> Result<(), ()> {
			// SHOULD DO: Refresh temperature readings for all ADCs once NTC reading is implemented
			// for adc_index in 0..AdcDevice::COUNT {
			// 	let adc = AdcDevice::from(adc_index);
			// 	match pressure_service_mutex.lock().await.refresh_rtd_reading(adc).await {
			// 		Err(e) => {
			// 			error!("Failed to read NTC on {:?}: {:?}", adc, e);
			// 		}
			// 		_ => {}
			// 	}
			// }

			// Delay the NTC measurement because it's not as critical as the pressures. We just need to read every once in a while
			Timer::after_millis(NTC_MEASUREMENT_INTERVAL).await;

			Ok(())
		})
		.await
		.unwrap();
}

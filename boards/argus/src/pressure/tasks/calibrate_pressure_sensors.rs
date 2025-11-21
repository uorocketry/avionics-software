use defmt::error;
use embassy_executor::task;
use embassy_futures::yield_now;
use strum::EnumCount;
use utils::types::AsyncMutex;

use crate::adc::types::AdcDevice;
use crate::pressure::service::PressureService;
use crate::state_machine::service::StateMachineWorker;
use crate::state_machine::types::States;

#[task]
pub async fn calibrate_pressure_sensors(
	mut worker: StateMachineWorker,
	pressure_service_mutex: &'static AsyncMutex<PressureService<{ AdcDevice::COUNT }>>,
) {
	worker
		.run_while(&[States::Calibrating], async |_| -> Result<(), ()> {
			let mut pressure_service = pressure_service_mutex.lock().await;
			// SHOULD DO: Refresh temperature readings for all ADCs once NTC reading is implemented
			// for adc_index in 0..AdcDevice::COUNT {
			// 	let adc = AdcDevice::from(adc_index);
			// 	match pressure_service.refresh_temperature_readings(adc).await {
			// 		Err(e) => {
			// 			error!("Failed to read NTC on {:?} during calibration: {:?}", adc, e);
			// 		}
			// 		_ => {}
			// 	}
			// }

			match pressure_service.calibrate().await {
				Ok(_) => {}
				Err(e) => error!("Pressure calibration failed: {:?}", e),
			}
			yield_now().await;
			Ok(())
		})
		.await
		.unwrap();
}

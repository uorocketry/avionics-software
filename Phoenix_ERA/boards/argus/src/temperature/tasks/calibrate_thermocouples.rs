use defmt::error;
use embassy_executor::task;
use embassy_futures::yield_now;
use strum::EnumCount;
use uor_utils::utils::types::AsyncMutex;

use crate::adc::types::AdcDevice;
use crate::state_machine::service::StateMachineWorker;
use crate::state_machine::types::States;
use crate::temperature::service::TemperatureService;

#[task]
pub async fn calibrate_thermocouples(
	mut worker: StateMachineWorker,
	temperature_service_mutex: &'static AsyncMutex<TemperatureService<{ AdcDevice::COUNT }>>,
) {
	worker
		.run_while(&[States::Calibrating], async |_| -> Result<(), ()> {
			let mut temperature_service = temperature_service_mutex.lock().await;
			for adc_index in 0..AdcDevice::COUNT {
				let adc = AdcDevice::from(adc_index);
				match temperature_service.refresh_rtd_reading(adc).await {
					Err(e) => {
						error!("Failed to read RTD on {:?} during calibration: {:?}", adc, e);
					}
					_ => {}
				}
			}

			match temperature_service.calibrate().await {
				Ok(_) => {}
				Err(e) => error!("Thermocouple calibration failed: {:?}", e),
			}
			yield_now().await;
			Ok(())
		})
		.await
		.unwrap();
}

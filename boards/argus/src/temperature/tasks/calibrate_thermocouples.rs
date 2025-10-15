use embassy_executor::task;
use embassy_futures::yield_now;
use embassy_time::Timer;
use strum::EnumCount;

use crate::adc::types::AdcDevice;
use crate::state_machine::service::StateMachineWorker;
use crate::state_machine::types::States;
use crate::temperature::service::TemperatureService;
use crate::utils::types::AsyncMutex;

#[task]
pub async fn calibrate_thermocouples(
	mut worker: StateMachineWorker,
	temperature_service_mutex: &'static AsyncMutex<TemperatureService<{ AdcDevice::COUNT }>>,
) {
	worker
		.run_while(States::Calibrating, async |_| -> Result<(), ()> {
			temperature_service_mutex.lock().await.calibrate().await;
			yield_now().await;
			Ok(())
		})
		.await
		.unwrap();
}

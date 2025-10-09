use csv::SerializeCSV;
use embassy_executor::task;
use heapless::format;
use strum::EnumCount;

use crate::adc::types::AdcDevice;
use crate::sd::service::SDCardService;
use crate::sd::types::{FileName, OperationScope};
use crate::state_machine::service::StateMachineWorker;
use crate::state_machine::types::States;
use crate::temperature::service::THERMOCOUPLE_READING_QUEUE;
use crate::temperature::types::{ThermocoupleChannel, ThermocoupleReading};
use crate::utils::types::AsyncMutex;

// Task for picking up the readings from the channel and logging them to the SD card
#[task]
pub async fn log_measurements(
	mut worker: StateMachineWorker,
	sd_card_service_mutex: &'static AsyncMutex<SDCardService>,
) {
	initialize_csv_files(sd_card_service_mutex).await;

	worker
		.run_while(States::Recording, async |_| -> Result<(), ()> {
			let (adc, channel, thermocouple_reading) = THERMOCOUPLE_READING_QUEUE.receive().await;
			let path = get_path_from_adc_and_channel(adc as usize, channel as usize);
			let line = thermocouple_reading.to_csv_line();
			SDCardService::enqueue_write(OperationScope::CurrentSession, path, line).await;
			Ok(())
		})
		.await
		.unwrap();
}

// Create the files and write the CSV headers before starting the logging loop
async fn initialize_csv_files(sd_card_service_mutex: &'static AsyncMutex<SDCardService>) {
	let mut sd_card_service = sd_card_service_mutex.lock().await;

	// Ignore because if the SD card isn't mounted we don't want to panic
	let _ = sd_card_service.ensure_session_created();
	for adc_index in 0..AdcDevice::COUNT {
		for channel in 0..ThermocoupleChannel::COUNT {
			let path = get_path_from_adc_and_channel(adc_index, channel);

			// Ignore because if the SD card isn't mounted we don't want to panic
			let _ = sd_card_service.write(OperationScope::CurrentSession, path, ThermocoupleReading::get_csv_header());
		}
	}
}

fn get_path_from_adc_and_channel(
	adc_index: usize,
	channel: usize,
) -> FileName {
	format!("T_{}_{}.csv", adc_index, channel).unwrap() as FileName
}

use defmt::info;
use embassy_executor::task;
use heapless::format;
use peripheral_services::serial::service::SerialService;
use strum::EnumCount;
use uor_utils::csv::SerializeCSV;
use uor_utils::messages::argus::envelope::envelope::Message;
use uor_utils::utils::types::AsyncMutex;

use crate::adc::types::AdcDevice;
use crate::pressure::service::PRESSURE_READING_QUEUE;
use crate::pressure::types::{PressureChannel, PressureReading};
use crate::sd::service::SDCardService;
use crate::sd::types::{FileName, OperationScope};
use crate::session::service::SessionService;
use crate::state_machine::service::StateMachineWorker;
use crate::state_machine::types::States;

// Task for picking up the readings from the channel and logging them to the SD card
#[task]
pub async fn log_measurements(
	mut worker: StateMachineWorker,
	serial_service_mutex: &'static AsyncMutex<SerialService>,
	sd_card_service_mutex: &'static AsyncMutex<SDCardService>,
	session_service: &'static AsyncMutex<SessionService>,
) {
	worker
		.run_once(&[States::Recording], async |_| -> Result<(), ()> {
			initialize_csv_files(sd_card_service_mutex, session_service).await;
			Ok(())
		})
		.await
		.unwrap();

	worker
		.run_while(&[States::Recording], async |_| -> Result<(), ()> {
			let pressure_reading = PRESSURE_READING_QUEUE.receive().await;
			let path = get_path_from_adc_and_channel(pressure_reading.adc_device as usize, pressure_reading.pressure_channel as usize);
			let line = pressure_reading.to_csv_line();
			SDCardService::enqueue_write(OperationScope::CurrentSession, path, line).await;
			let _ = serial_service_mutex
				.lock()
				.await
				.write_envelope_message(Message::PressureReading(pressure_reading.to_protobuf()))
				.await;
			Ok(())
		})
		.await
		.unwrap();
}

// Create the files and write the CSV headers before starting the logging loop
async fn initialize_csv_files(
	sd_card_service_mutex: &'static AsyncMutex<SDCardService>,
	session_service: &'static AsyncMutex<SessionService>,
) {
	// Ensure session is set. Ignore if it errors like SD card not mounted, etc.
	let _ = session_service.lock().await.ensure_session().await;

	info!("Initializing CSV files for measurement logging.");
	let mut sd_card_service = sd_card_service_mutex.lock().await;
	for adc_index in 0..AdcDevice::COUNT {
		for channel in 0..PressureChannel::COUNT {
			let path = get_path_from_adc_and_channel(adc_index, channel);

			// Ignore because if the SD card isn't mounted we don't want to panic
			let _ = sd_card_service.write(OperationScope::CurrentSession, path, PressureReading::get_csv_header());
		}
	}
}

fn get_path_from_adc_and_channel(
	adc_index: usize,
	channel: usize,
) -> FileName {
	format!("P_{}_{}.csv", adc_index, channel).unwrap() as FileName
}

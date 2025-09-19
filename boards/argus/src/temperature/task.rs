// Embassy tasks that need to run for temperature measurement and logging
use core::fmt::Write;

use argus::config::{AdcDevice, ADC_COUNT};
use argus::sd::csv::types::SerializeCSV;
use argus::sd::service::SDCardService;
use argus::sd::types::{FileName, OperationScope};
use argus::state_machine::service::StateMachineWorker;
use argus::state_machine::types::{Events, States};
use argus::temperature::config::{ThermocoupleChannel, CHANNEL_COUNT};
use argus::temperature::service::TemperatureService;
use argus::temperature::types::{ThermocoupleReading, ThermocoupleReadingChannel};
use argus::utils::types::AsyncMutex;
use defmt::debug;

// A channel for buffering the temperature readings and decoupling the logging to sd task from the measurement task
static THERMOCOUPLE_READING_CHANNEL: ThermocoupleReadingChannel = ThermocoupleReadingChannel::new();

// Task that iterates through the ADCs and channels, measures the temperature, and enqueues the readings to a channel
#[embassy_executor::task]
pub async fn measure_and_enqueue_temperature_readings(
	mut worker: StateMachineWorker,
	temperature_service_mutex: &'static AsyncMutex<TemperatureService>,
) {
	// Configure the ADCs for temperature measurement
	temperature_service_mutex.lock().await.configure_adcs().await.unwrap();

	worker
		.run_while(States::Recording, async |_| -> Result<(), ()> {
			let mut temperature_service = temperature_service_mutex.lock().await;

			for adc_index in 0..ADC_COUNT {
				for channel_index in 0..CHANNEL_COUNT {
					let adc = AdcDevice::from(adc_index);
					let channel = ThermocoupleChannel::from(channel_index);
					let data = temperature_service.read_thermocouple(adc, channel).await;
					match data {
						Ok(data) => {
							debug!("ADC {} Channel {}: {}", adc, channel, data);
							THERMOCOUPLE_READING_CHANNEL.send((adc, channel, data)).await;
						}
						Err(error) => {
							debug!("Error reading ADC {} Channel {}: {:?}", adc, channel, error);
							continue;
						}
					}
				}
			}
			Ok(())
		})
		.await
		.unwrap();
}

// Task for picking up the readings from the channel and logging them to the SD card
#[embassy_executor::task]
pub async fn log_thermocouple_reading_to_sd_card(
	mut worker: StateMachineWorker,
	sd_service_mutex: &'static AsyncMutex<SDCardService>,
) {
	initialize_csv_files(sd_service_mutex).await;

	worker
		.run_while(States::Recording, async |_| -> Result<(), ()> {
			let (adc, channel, thermocouple_reading) = THERMOCOUPLE_READING_CHANNEL.receive().await;
			let path = get_path_from_adc_and_channel(adc as usize, channel as usize);
			let line = thermocouple_reading.to_csv_line();
			SDCardService::enqueue_write(OperationScope::CurrentSession, path, line).await;
			Ok(())
		})
		.await
		.unwrap();
}

// Create the files and write the CSV headers before starting the logging loop
async fn initialize_csv_files(sd_service_mutex: &'static AsyncMutex<SDCardService>) {
	let mut sd_service = sd_service_mutex.lock().await;

	// Ignore because if the SD card isn't mounted we don't want to panic
	let _ = sd_service.ensure_session_created();
	for adc_index in 0..ADC_COUNT {
		for channel in 0..CHANNEL_COUNT {
			let path = get_path_from_adc_and_channel(adc_index, channel);

			// Ignore because if the SD card isn't mounted we don't want to panic
			let _ = sd_service.write(OperationScope::CurrentSession, path, ThermocoupleReading::get_header());
		}
	}
}

fn get_path_from_adc_and_channel(
	adc_index: usize,
	channel: usize,
) -> FileName {
	let mut path = FileName::new();
	write!(path, "T_{}_{}.csv", adc_index, channel).unwrap();
	path
}

use core::fmt::Write;

use defmt::debug;
use serde_csv_core::Writer;

use crate::adc::service::AdcService;
use crate::sd::csv::types::SerializeCSV;
use crate::sd::service::SDCardService;
use crate::sd::types::{FilePath, OperationScope};
use crate::temperature::service::TemperatureAdcService;
use crate::temperature::types::{ThermocoupleChannel, ThermocoupleReading, ThermocoupleReadingChannel};
use crate::utils::types::AsyncMutex;

// A channel for buffering the temperature readings and decoupling the logging to sd task from the measurement task
static THERMOCOUPLE_READING_CHANNEL: ThermocoupleReadingChannel = ThermocoupleReadingChannel::new();

#[embassy_executor::task]
pub async fn measure_and_enqueue_temperature_readings(adc_service_mutex: &'static AsyncMutex<AdcService>) {
	// Configure the ADCs for temperature measurement
	adc_service_mutex.lock().await.configure().await.unwrap();

	loop {
		let mut service = adc_service_mutex.lock().await;
		for adc_index in 0..service.drivers.len() {
			for channel in 0..ThermocoupleChannel::len() {
				// // For now for the sake of testing only read from ADC 2 Channel 2
				// if adc_index != 1 && channel != 1 {
				// 	continue;
				// }

				let channel = ThermocoupleChannel::from(channel);
				let data = service.read_thermocouple(adc_index, channel).await.unwrap();

				debug!("ADC {} Channel {}: {}", adc_index, channel as usize, data);
				THERMOCOUPLE_READING_CHANNEL.send((adc_index, channel, data)).await;
			}
		}
	}
}

#[embassy_executor::task]
pub async fn log_temperature_reading_to_sd_card(
	adc_service_mutex: &'static AsyncMutex<AdcService>,
	sd_card_service_mutex: &'static AsyncMutex<SDCardService>,
) {
	initialize_csv_files(adc_service_mutex, sd_card_service_mutex).await;

	// Continuously write the readings to sd as they come in from the channel
	let mut csv_writer = Writer::new();
	loop {
		let (adc_index, channel, reading) = THERMOCOUPLE_READING_CHANNEL.receive().await;
		let path = get_path_from_adc_and_channel(adc_index, channel as usize);
		let line = reading.to_csv_line(&mut csv_writer);
		SDCardService::enqueue_write(OperationScope::CurrentSession, path, line).await;
	}
}

// Create the files and write the CSV headers before starting the logging loop
async fn initialize_csv_files(
	adc_service_mutex: &'static AsyncMutex<AdcService>,
	sd_card_service_mutex: &'static AsyncMutex<SDCardService>,
) {
	let mut sd_card_service = sd_card_service_mutex.lock().await;

	// Ignore because if the SD card isn't mounted we don't want to panic
	let _ = sd_card_service.ensure_session_created();
	for adc_index in 0..adc_service_mutex.lock().await.drivers.len() {
		for channel in 0..ThermocoupleChannel::len() {
			let path = get_path_from_adc_and_channel(adc_index, channel);

			// Ignore because if the SD card isn't mounted we don't want to panic
			let _ = sd_card_service.write(OperationScope::CurrentSession, path, ThermocoupleReading::get_header());
		}
	}
}

fn get_path_from_adc_and_channel(
	adc_index: usize,
	channel: usize,
) -> FilePath {
	let mut path = FilePath::new();
	write!(path, "T_{}_{}.csv", adc_index, channel).unwrap();
	path
}

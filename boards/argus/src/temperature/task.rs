use defmt::info;
use embassy_time::Timer;
use heapless::String;

use crate::adc::driver::types::AnalogChannel;
use crate::adc::service::AdcService;
use crate::temperature::service::TemperatureAdcService;
use crate::utils::types::AsyncMutex;

#[embassy_executor::task]
pub async fn temperature_task(temperature_adc_service: &'static AsyncMutex<AdcService>) {
	temperature_adc_service
		.lock()
		.await
		.configure_for_temperature_measurement()
		.await
		.unwrap();
	loop {
		let mut service = temperature_adc_service.lock().await;
		let data = service.read_thermocouple(1, 1).await.unwrap();
		info!("Thermocouple Reading: {:?}", data);
		// let mut data: [[f32; 10]; 2] = [[0.0; 10]; 2];
		// for adc_index in 0..2 {
		// 	for channel_index in 0..10 {
		// 		data[adc_index][channel_index] = service.drivers[adc_index].read_single_ended(AnalogChannel::from(channel_index as u8)).await.unwrap();
		// 	}
		// }
		// info!("Reading: {:?}", data);

		// let (device_id, revision_id) = service.drivers[1].get_id_and_revision().await.unwrap();
		// info!("Device ID: {}, Revision ID: {}", device_id, revision_id);
	}
}

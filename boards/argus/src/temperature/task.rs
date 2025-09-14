use defmt::info;
use crate::adc::service::AdcService;
use crate::temperature::service::TemperatureAdcService;
use crate::utils::types::AsyncMutex;

#[embassy_executor::task]
pub async fn temperature_task(
	temperature_adc_service: &'static AsyncMutex<AdcService>,
)
{
	temperature_adc_service.lock().await.configure_for_temperature_measurement().await.unwrap();
	loop {
		let data = temperature_adc_service.lock().await.read_thermocouple(0, 0).await.unwrap();
		info!("Thermocouple Reading: {:?}", data);
	}
}

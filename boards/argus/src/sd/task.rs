use defmt::{debug, error};
use utils::types::AsyncMutex;

use crate::led_indicator::service::LedIndicatorService;
use crate::sd::service::{SDCardService, SD_CARD_WRITE_QUEUE};

#[embassy_executor::task]
pub async fn sd_card_task(
	sd_card_service_mutex: &'static AsyncMutex<SDCardService>,
	led_indicator_service_mutex: &'static AsyncMutex<LedIndicatorService<2>>,
) {
	debug!("Starting SD card write loop.");
	loop {
		let (scope, path, line) = SD_CARD_WRITE_QUEUE.receiver().receive().await;
		if let Err(error) = sd_card_service_mutex.lock().await.write(scope, path, line) {
			error!("Could not write to SD card: {}", error);
			continue;
		} else {
			led_indicator_service_mutex.lock().await.blink(0).await;
		}
	}
}

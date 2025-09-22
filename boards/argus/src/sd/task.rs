use defmt::{debug, error};

use crate::sd::service::{SDCardService, SD_CARD_WRITE_QUEUE};
use crate::utils::types::AsyncMutex;

#[embassy_executor::task]
pub async fn sd_card_task(sd_card_service_mutex: &'static AsyncMutex<SDCardService>) {
	if let Err(error) = sd_card_service_mutex.lock().await.ensure_session_created() {
		error!("Could not create session directory: {:?}", error);
	}

	debug!("Starting SD card write loop.");
	loop {
		let (scope, path, line) = SD_CARD_WRITE_QUEUE.receiver().receive().await;
		if let Err(error) = sd_card_service_mutex.lock().await.write(scope, path, line) {
			error!("Could not write to SD card: {}", error);
			continue;
		}
	}
}

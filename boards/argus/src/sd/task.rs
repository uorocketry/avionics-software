use core::fmt::Write;
use core::str::FromStr;

use argus::sd::service::SDCardService;
use argus::sd::types::OperationScope;
use argus::utils::types::AsyncMutex;
use heapless::String;

#[embassy_executor::task]
pub async fn sd_card_task(sd_service: &'static AsyncMutex<SDCardService>) {
	SDCardService::ensure_task(sd_service).await;
}

// SHOULD DO: move this to unit tests
#[embassy_executor::task]
pub async fn test_sd_card() {
	for i in 0..5 {
		let path = String::from_str("test.txt").unwrap();
		let mut message: String<255> = String::new();
		write!(message, "Hello, world {}! \n", i).unwrap();
		SDCardService::enqueue_write(OperationScope::CurrentSession, path, message).await;
	}
}

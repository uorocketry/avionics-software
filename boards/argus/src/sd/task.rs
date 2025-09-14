use core::fmt::Write;
use core::str::FromStr;

use heapless::String;

use crate::sd::service::SDCardService;
use crate::sd::types::OperationScope;
use crate::utils::types::AsyncMutex;

#[embassy_executor::task]
pub async fn sd_card_task(sd_card_service: &'static AsyncMutex<SDCardService>) {
	SDCardService::ensure_task(sd_card_service).await;
}

// SHOULD DO: move this to unit tests
#[embassy_executor::task]
pub async fn test_sd_card() {
	for i in 0..5 {
		let mut message: String<255> = String::new();
		write!(message, "Hello, world {}! \n", i).unwrap();

		SDCardService::enqueue_write(OperationScope::CurrentSession, String::from_str("test.txt").unwrap(), message).await;
	}
}

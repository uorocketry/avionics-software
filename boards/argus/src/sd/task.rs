use crate::sd::service::SDCardService;
use crate::utils::types::AsyncMutex;

#[embassy_executor::task]
pub async fn sd_card_task(
	sd_card_service: &'static AsyncMutex<SDCardService>,
)
{
	SDCardService::ensure_task(sd_card_service).await;
}

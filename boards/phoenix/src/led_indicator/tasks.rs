use crate::{
	led_indicator::{
		service::LedIndicatorService,
	},
	utils::types::AsyncMutex,
};

#[embassy_executor::task]
pub async fn cycle_leds(leds: &'static AsyncMutex<LedIndicatorService<2>>) {
	loop {
		let mut guard = leds.lock().await;
		guard.blink(0).await;
		guard.blink(1).await;
	}
}

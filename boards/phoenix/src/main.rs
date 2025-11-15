#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use defmt::info;
use embassy_executor::Spawner;
use panic_probe as _;
use phoenix::{
	sound::service::SoundService,
	utils::{
		hal::{HEAP, configure_hal},
		types::AsyncMutex,
	},
};
use static_cell::StaticCell;

/// To change the pin used for sound, see [phoenix::sound::types]
static SOUND_SERVICE: StaticCell<AsyncMutex<SoundService>> = StaticCell::new();
#[cfg(feature = "music")]
static MUSIC_SERVICE: StaticCell<AsyncMutex<phoenix::music::service::MusicService>> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
	info!("Starting up...");
	let p = configure_hal();
	info!("Heap usage: {} bytes", HEAP.used());

	let sound = SOUND_SERVICE.init(AsyncMutex::new(SoundService::new(p.TIM3, p.PC6)));

	#[cfg(feature = "music")]
	{
		use defmt::error;
		use phoenix::music::{service::MusicService, tasks::play_music_forever};
		let music = MUSIC_SERVICE.init(AsyncMutex::new(MusicService::new(sound)));
		match spawner.spawn(play_music_forever(music)) {
			Ok(_) => (),
			Err(e) => error!("Could not spawn music task: {}", e),
		}
	}
}

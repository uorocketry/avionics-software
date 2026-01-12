//! Playback of some simple songs with the buzzer
use embassy_time::Timer;

use crate::{
	music::types::{Melody, Note},
	sound::service::SoundService,
	utils::types::AsyncMutex,
};

pub struct MusicService {
	sound_service: &'static AsyncMutex<SoundService>,
}

impl MusicService {
	pub fn new(sound_service: &'static AsyncMutex<SoundService>) -> Self {
		Self { sound_service }
	}

	pub async fn play_song(
		&self,
		song: Melody,
		tempo: f32,
	) {
		let beat_length = 60_000f32 / tempo;
		let mut sound = self.sound_service.lock().await;

		for (note, length) in song {
			let duration = (length * beat_length) as u64;
			match note {
				Note::Pitch(freq) => sound.play_pitch(*freq as u32, duration).await,
				Note::Rest => Timer::after_millis(duration).await,
			}
		}
	}
}

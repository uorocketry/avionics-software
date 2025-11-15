use crate::{
	music::{
		service::MusicService,
		songs::{PLAYLIST, TEMPO},
	},
	utils::types::AsyncMutex,
};

#[embassy_executor::task]
pub async fn play_music_forever(music: &'static AsyncMutex<MusicService>) {
	for song in PLAYLIST.iter().cycle() {
		let guard = music.lock().await;
		guard.play_song(song, TEMPO).await;
	}
}

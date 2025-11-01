use embassy_stm32::timer::GeneralInstance4Channel;

use crate::{
	music::{service::MusicService, songs::PLAYLIST},
	utils::types::AsyncMutex,
};

// FIXME: Embassy tasks don't allow generics, but we kinda need some if we don't want to be forced
// into hardcoding a specific pin in the library code.
//
// Potential fix could involve creating a macro which generates the task and using the macro at the
// top level of `main.rs`

// #[embassy_executor::task]
// async fn play_music_forever(music: &'static AsyncMutex<MusicService<impl GeneralInstance4Channel>>) {
// 	for song in PLAYLIST {}
// }

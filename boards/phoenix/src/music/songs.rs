//! A bunch of songs to play with the buzzer.

// TODO: Implement a DSL macro to easily add songs to the module.
// macro_rules! add_song {
//     (eval ) => {
//
//     };
// }

// NOTE: Here are some of the previous songs, written in an alternate format.

use crate::music::types::Melody;

pub const PLAYLIST: [Melody; 0] = [];

// pub const MARIO_MELODY: Melody = &[
// 	(E5, 1.0),
// 	(E5, 1.0),
// 	(Rest, 1.0),
// 	(E5, 1.0),
// 	(Rest, 1.0),
// 	(C5, 1.0),
// 	(E5, 1.0),
// 	(Rest, 1.0),
// 	(G5, 2.0),
// 	(Rest, 2.0),
// 	(G4, 2.0),
// 	(Rest, 2.0),
// ];
//
// /// Melody for "Twinkle, Twinkle, Little Star"
// pub const TWINKLE_MELODY: Melody = &[
// 	(C4, 1.0),
// 	(C4, 1.0),
// 	(G4, 1.0),
// 	(G4, 1.0),
// 	(A4, 1.0),
// 	(A4, 1.0),
// 	(G4, 2.0),
// 	(Rest, 0.5),
// 	(F4, 1.0),
// 	(F4, 1.0),
// 	(E4, 1.0),
// 	(E4, 1.0),
// 	(D4, 1.0),
// 	(D4, 1.0),
// 	(C4, 2.0),
// ];

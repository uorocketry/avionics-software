//! A bunch of songs to play with the buzzer.
use Note::{Pitch, Rest};

use crate::music::types::Melody;
use crate::music::types::Note;

pub const A4: Note = Pitch(440);
pub const C4: Note = Pitch(262);
pub const D4: Note = Pitch(294);
pub const E4: Note = Pitch(330);
pub const F4: Note = Pitch(349);
pub const G4: Note = Pitch(392);
pub const C5: Note = Pitch(523);
pub const E5: Note = Pitch(659);
pub const G5: Note = Pitch(784);

pub const TEMPO: f32 = 120.0;
pub const PLAYLIST: [Melody; 2] = [MARIO_MELODY, TWINKLE_MELODY];

pub const MARIO_MELODY: Melody = &[
	(E5, 1.0),
	(E5, 1.0),
	(Rest, 1.0),
	(E5, 1.0),
	(Rest, 1.0),
	(C5, 1.0),
	(E5, 1.0),
	(Rest, 1.0),
	(G5, 2.0),
	(Rest, 2.0),
	(G4, 2.0),
	(Rest, 2.0),
];

/// Melody for "Twinkle, Twinkle, Little Star"
pub const TWINKLE_MELODY: Melody = &[
	(C4, 1.0),
	(C4, 1.0),
	(G4, 1.0),
	(G4, 1.0),
	(A4, 1.0),
	(A4, 1.0),
	(G4, 2.0),
	(Rest, 0.5),
	(F4, 1.0),
	(F4, 1.0),
	(E4, 1.0),
	(E4, 1.0),
	(D4, 1.0),
	(D4, 1.0),
	(C4, 2.0),
];

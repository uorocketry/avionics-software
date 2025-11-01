/// A note, representing either a pitch (with a frequency in Hz), or a rest.
// NOTE: Derive Copy since it's basically just a cheap u32 copy
#[derive(Clone, Copy)]
pub enum Note {
	Pitch(u32),
	Rest,
}

/// An array of notes with a length, in beats.
pub type Melody = &'static [(Note, f32)];

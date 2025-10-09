/// Preset Gain values from ADS126x datasheet
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Gain {
	G1 = 0b000,
	G2 = 0b001,
	G4 = 0b010,
	G8 = 0b011,
	G16 = 0b100,
	G32 = 0b101,
}

impl Gain {
	pub fn to_multiplier(&self) -> f32 {
		match self {
			Gain::G1 => 1.0,
			Gain::G2 => 2.0,
			Gain::G4 => 4.0,
			Gain::G8 => 8.0,
			Gain::G16 => 16.0,
			Gain::G32 => 32.0,
		}
	}
}

/// Preset Gain values from ADS126x datasheet
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Gain {
	G1 = 0b000,
	G2 = 0b001,
	G4 = 0b010,
	G8 = 0b011,
	G16 = 0b100,
	G32 = 0b101,
}

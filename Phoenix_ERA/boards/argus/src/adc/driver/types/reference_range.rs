/// Defines the reference voltage for the ADC.
/// This defines the full-scale-differential input range = VREFP - VREFN / Gain
#[derive(Copy, Clone, Debug)]
pub enum ReferenceRange {
	Avdd,        // REFP = Avdd, REFN = Avss
	Internal2_5, // REFP = Internal 2.5V REFN = Avss
}
impl ReferenceRange {
	pub fn to_volts(&self) -> f32 {
		match self {
			ReferenceRange::Avdd => 5.0,
			ReferenceRange::Internal2_5 => 2.5,
		}
	}
}

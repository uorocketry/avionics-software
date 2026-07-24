#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AnalogChannel {
	AIN0 = 0,
	AIN1 = 1,
	AIN2 = 2,
	AIN3 = 3,
	AIN4 = 4,
	AIN5 = 5,
	AIN6 = 6,
	AIN7 = 7,
	AIN8 = 8,
	AIN9 = 9,
	AINCOM = 10,
}

impl AnalogChannel {
	pub fn from(value: u8) -> Self {
		match value {
			0 => AnalogChannel::AIN0,
			1 => AnalogChannel::AIN1,
			2 => AnalogChannel::AIN2,
			3 => AnalogChannel::AIN3,
			4 => AnalogChannel::AIN4,
			5 => AnalogChannel::AIN5,
			6 => AnalogChannel::AIN6,
			7 => AnalogChannel::AIN7,
			8 => AnalogChannel::AIN8,
			9 => AnalogChannel::AIN9,
			10 => AnalogChannel::AINCOM,
			_ => panic!("Invalid AnalogChannel value: {}", value),
		}
	}
}

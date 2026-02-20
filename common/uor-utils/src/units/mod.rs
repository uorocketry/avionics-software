use defmt::{Format, info};

// TODO: Can probably make these types instead of structs
#[derive(Default, Format)]
pub struct Temperature {
	// Internal value is celsius with decimal points stripped (2103 vs 21.03)
	internal: i64,
}
// TODO: Cache value after first request (after calculating celsiusf, cache the value for future reads)
// TODO: Make floating point variants a feature
// TODO: Make a derive macro for the metric standard (autogenerate functions for milli, centi, deca, kilo, etc)
impl Temperature {
	pub fn new(degrees_celsius: i64) -> Self {
		Temperature { internal: degrees_celsius }
	}

	pub fn celsius(&self) -> i64 {
		self.internal.clone() / 100
	}

	pub fn fahrenheit(&self) -> i64 {
		((self.internal.clone() / 100 * 9 / 5) + 32)
	}

	pub fn kelvin(&self) -> i64 {
		self.internal.clone() / 100 + 273
	}

	pub fn raw_celsius(&self) -> i64 {
		self.internal.clone()
	}

	pub fn raw_fahrenheit(&self) -> i64 {
		(self.internal.clone() * 9 / 5) + 3200
	}

	pub fn raw_kelvin(&self) -> i64 {
		self.internal.clone() + 27300
	}

	pub fn fcelsius(&self) -> f64 {
		self.internal.clone() as f64 / 100.0
	}

	pub fn ffahrenheit(&self) -> f64 {
		(self.internal.clone() as f64 / 100.0 * 9.0 / 5.0) + 32.0
	}

	pub fn fkelvin(&self) -> f64 {
		(self.internal.clone() as f64) / 100.0 + 273.15
	}
}

#[derive(Default, Format)]
pub struct Pressure {
	// Internal is the pressure in mbar with the decimal points stripped (100009 =  1000.09 mbar)
	internal: i64,
}
impl Pressure {
	pub fn new(pressure_mbar: i64) -> Self {
		Pressure { internal: pressure_mbar }
	}

	pub fn mbar(&self) -> i64 {
		self.internal / 100
	}

	pub fn bar(&self) -> i64 {
		self.internal / 100 / 1000
	}

	pub fn psi(&self) -> i64 {
		self.internal / 100 * 15
	}

	pub fn fmbar(&self) -> f64 {
		self.internal as f64 / 100.0
	}

	pub fn fbar(&self) -> f64 {
		self.internal as f64 / 100.0 / 1000.0
	}

	pub fn fpsi(&self) -> f64 {
		self.internal as f64 / 100.0 * 15.0
	}
}

#[derive(Default, Format)]
pub struct Altitude {
	// Internal value is in floating point feet
	internal: f64,
}

impl Altitude {
	pub fn new(feet: f64) -> Self {
		Altitude { internal: feet }
	}

	pub fn feet(&self) -> i64 {
		self.internal as i64
	}

	pub fn yard(&self) -> i64 {
		self.internal as i64 / 3
	}

	pub fn meters(&self) -> i64 {
		self.internal as i64 / 3
	}

	pub fn kilometers(&self) -> i64 {
		self.internal as i64 / 3000
	}

	pub fn ffeet(&self) -> f64 {
		self.internal
	}

	pub fn fyard(&self) -> f64 {
		self.internal / 3.0
	}

	pub fn fmeters(&self) -> f64 {
		self.internal / 3.281
	}

	pub fn fkilometers(&self) -> f64 {
		self.internal / 3.281 / 1000.0
	}
}

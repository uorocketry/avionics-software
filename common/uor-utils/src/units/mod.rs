use core::{f64::consts::PI, ops::Add};

use defmt::{Format, info};

use crate::linear_algebra::types::Zero;

// TODO: Can probably make these types instead of structs
#[derive(Default, Format, Clone)]
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

impl core::ops::Add<Temperature> for Temperature {
	type Output = Temperature;

	fn add(
		self,
		rhs: Temperature,
	) -> Self::Output {
		Temperature {
			internal: self.internal + rhs.internal,
		}
	}
}

impl Zero for Temperature {
	fn zero() -> Self {
		Self { internal: 0 }
	}
}

#[derive(Default, Format, Clone)]
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

impl core::ops::Add<Pressure> for Pressure {
	type Output = Pressure;

	fn add(
		self,
		rhs: Pressure,
	) -> Self::Output {
		Pressure {
			internal: self.internal + rhs.internal,
		}
	}
}

impl Zero for Pressure {
	fn zero() -> Self {
		Self { internal: 0 }
	}
}

#[derive(Default, Format, Clone)]
pub struct Distance {
	// Internal value is in floating point feet
	internal: f64,
}

impl Distance {
	pub fn new(feet: f64) -> Self {
		Distance { internal: feet }
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

impl core::ops::Add<Distance> for Distance {
	type Output = Distance;

	fn add(
		self,
		rhs: Distance,
	) -> Self::Output {
		Distance {
			internal: self.internal + rhs.internal,
		}
	}
}

impl Zero for Distance {
	fn zero() -> Self {
		Self { internal: 0.0 }
	}
}
#[derive(Default, Format, Clone)]

pub struct Angle {
	// Internal value is in floating radians
	internal: f64,
}

impl Angle {
	pub fn new(radians: f64) -> Self {
		Angle { internal: radians }
	}

	pub fn degrees(&self) -> i64 {
		(self.internal * 180.0 / PI) as i64
	}

	pub fn fdegrees(&self) -> f64 {
		self.internal * 180.0 / PI
	}

	pub fn radians(&self) -> i64 {
		self.internal as i64
	}

	pub fn fradians(&self) -> f64 {
		self.internal
	}
}

impl core::ops::Add<Angle> for Angle {
	type Output = Angle;

	fn add(
		self,
		rhs: Angle,
	) -> Self::Output {
		Angle {
			internal: self.internal + rhs.internal,
		}
	}
}
impl Zero for Angle {
	fn zero() -> Self {
		Self { internal: 0.0 }
	}
}

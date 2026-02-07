pub struct VectorR3 {
	internal: [i64; 3],
}

impl VectorR3 {
	pub fn new(
		i: i64,
		j: i64,
		k: i64,
	) -> Self {
		VectorR3 { internal: [i, j, k] }
	}

	pub fn zero_vector() -> VectorR3 {
		VectorR3::new(0, 0, 0)
	}
}

// Do the basic implementations for VectorR3 to be a vector space (Addition)
impl core::ops::Add<VectorR3> for VectorR3 {
	type Output = VectorR3;

	fn add(
		self,
		rhs: VectorR3,
	) -> Self::Output {
		VectorR3::new(
			self.internal[0] + rhs.internal[0],
			self.internal[1] + rhs.internal[1],
			self.internal[2] + rhs.internal[2],
		)
	}
}
// Do the basic implementations for VectorR3 to be a vector space (Scalar Multiplication)
impl core::ops::Mul<i64> for VectorR3 {
	type Output = VectorR3;

	fn mul(
		self,
		rhs: i64,
	) -> Self::Output {
		VectorR3::new(self.internal[0] * rhs, self.internal[1] * rhs, self.internal[2] * rhs)
	}
}

pub struct VectorR3F {
	internal: [f64; 3],
}

impl VectorR3F {
	pub fn new(
		i: f64,
		j: f64,
		k: f64,
	) -> Self {
		VectorR3F { internal: [i, j, k] }
	}

	pub fn zero_vector() -> VectorR3F {
		VectorR3F::new(0.0, 0.0, 0.0)
	}
}
// Do the basic implementations for VectorR3 to be a vector space (Addition)
impl core::ops::Add<VectorR3F> for VectorR3F {
	type Output = VectorR3F;

	fn add(
		self,
		rhs: VectorR3F,
	) -> Self::Output {
		VectorR3F::new(
			self.internal[0] + rhs.internal[0],
			self.internal[1] + rhs.internal[1],
			self.internal[2] + rhs.internal[2],
		)
	}
}
// Do the basic implementations for VectorR3 to be a vector space (Scalar Multiplication)
impl core::ops::Mul<f64> for VectorR3F {
	type Output = VectorR3F;

	fn mul(
		self,
		rhs: f64,
	) -> Self::Output {
		VectorR3F::new(self.internal[0] * rhs, self.internal[1] * rhs, self.internal[2] * rhs)
	}
}
impl core::ops::Mul<VectorR3F> for VectorR3F {
	type Output = VectorR3F;

	fn mul(
		self,
		rhs: VectorR3F,
	) -> Self::Output {
		VectorR3F::new(
			self.internal[0] * rhs.internal[0],
			self.internal[1] * rhs.internal[1],
			self.internal[2] * rhs.internal[2],
		)
	}
}

impl core::convert::From<VectorR3> for VectorR3F {
	fn from(value: VectorR3) -> Self {
		let (i, j, k) = value.internal.into();

		VectorR3F::new(i as f64, j as f64, k as f64)
	}
}

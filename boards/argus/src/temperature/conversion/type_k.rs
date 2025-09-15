// BIG NOTE: This file has been vibe coded cuz I was tired and couldn't read so this needs to be revisited!

pub fn compensated_emf_to_temperature(
	voltage_in_millivolts: f32,
	cold_junction_temperature_in_celsius: f32,
) -> Option<f32> {
	let cold_junction_emf_in_millivolts = temperature_to_emf(cold_junction_temperature_in_celsius)?;
	let total_emf_millivolts = voltage_in_millivolts + cold_junction_emf_in_millivolts;
	emf_to_temperature(total_emf_millivolts)
}

pub fn temperature_to_emf(temperature_in_celsius: f32) -> Option<f32> {
	// Valid temperature span for Type K reference function per ITS‑90: −270 °C to 1372 °C.
	if !(-270.0..=1372.0).contains(&temperature_in_celsius) {
		return None;
	}

	if temperature_in_celsius < 0.0 {
		// Range: −270 °C to 0 °C
		// E = Σ c_i * t^i (E in mV)
		const C: [f32; 11] = [
			0.000000000000E+00,
			0.394501280250E-01,
			0.236223735980E-04,
			-0.328589067840E-06,
			-0.499048287770E-08,
			-0.675090591730E-10,
			-0.574103274280E-12,
			-0.310888728940E-14,
			-0.104516093650E-16,
			-0.198892668780E-19,
			-0.163226974860E-22,
		];
		Some(horner(&C, temperature_in_celsius))
	} else {
		// Range: 0 °C to 1372 °C
		// E = Σ c_i * t^i + α0 * exp( α1 * (t − 126.9686)^2 )  (E in mV)
		const C: [f32; 10] = [
			-0.176004136860E-01,
			0.389212049750E-01,
			0.185587700320E-04,
			-0.994575928740E-07,
			0.318409457190E-09,
			-0.560728448890E-12,
			0.560750590590E-15,
			-0.320207200030E-18,
			0.971511471520E-22,
			-0.121047212750E-25,
		];
		const ALPHA_0: f32 = 0.118597600000E+00;
		const ALPHA_1: f32 = -0.118343200000E-03;
		const A2: f32 = 0.126968600000E+03; // 126.9686 °C
		let poly_term = horner(&C, temperature_in_celsius);
		let dt = temperature_in_celsius - A2;
		let exp_term = ALPHA_0 * libm::expf(ALPHA_1 * dt * dt);
		Some(poly_term + exp_term)
	}
}

pub fn emf_to_temperature(emf_millivolts: f32) -> Option<f32> {
	// Piecewise EMF ranges for the ITS‑90 inverse fit (mV):
	//  −5.891 ≤ E < 0.000   →  −200 °C to 0 °C
	//   0.000 ≤ E < 20.644  →     0 °C to 500 °C
	//  20.644 ≤ E ≤ 54.886  →   500 °C to 1372 °C

	if emf_millivolts < -5.891 || emf_millivolts > 54.886 {
		return None;
	}

	if emf_millivolts < 0.0 {
		const D: [f32; 10] = [
			0.0000000E+00,
			2.5173462E+01,
			-1.1662878E+00,
			-1.0833638E+00,
			-8.9773540E-01,
			-3.7342377E-01,
			-8.6632643E-02,
			-1.0450598E-02,
			-5.1920577E-04,
			0.0000000E+00,
		];
		Some(horner(&D, emf_millivolts))
	} else if emf_millivolts < 20.644 {
		const D: [f32; 10] = [
			0.000000E+00,
			2.508355E+01,
			7.860106E-02,
			-2.503131E-01,
			8.315270E-02,
			-1.228034E-02,
			9.804036E-04,
			-4.413030E-05,
			1.057734E-06,
			-1.052755E-08,
		];
		Some(horner(&D, emf_millivolts))
	} else {
		const D: [f32; 7] = [
			-1.318058E+02,
			4.830222E+01,
			-1.646031E+00,
			5.464731E-02,
			-9.650715E-04,
			8.802193E-06,
			-3.110810E-08,
		];
		Some(horner(&D, emf_millivolts))
	}
}

#[inline(always)]
fn horner(
	coefficients: &[f32],
	x: f32,
) -> f32 {
	let mut acc = 0.0f32;
	// Evaluate highest order first (reverse) for numerical stability.
	for &c in coefficients.iter().rev() {
		acc = acc * x + c;
	}
	acc
}

fn pow(
	base: f32,
	exp: i32,
) -> f32 {
	if exp < 0 {
		return 1.0 / pow(base, -exp);
	}

	let mut result = 1.0;
	for _ in 0..exp {
		result *= base;
	}
	return result;
}

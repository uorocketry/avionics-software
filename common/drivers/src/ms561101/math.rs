use defmt::info;

use crate::ms561101::config::CalibrationData;
// TODO: Switch all divs by 2^n to bitwise shift. Be careful, as shifting can add or strip negativity due to the nature of how signed values are stored

// 	ALL OF THE FOLLOWING ARE FOR SITUATIONS WHERE THE AMBIENT IS <20°C
pub fn calculate_temperature_difference(
	temperature_value: &u32,
	calibration_data: &CalibrationData,
) -> i64 {
	temperature_value.clone() as i64 - ((calibration_data.tref as i64) * 2_i64.pow(8))
}
// Units are degrees celsius, 0.01 resolution
pub fn calculate_actual_temperature(
	temperature_value: &u32,
	calibration_data: &CalibrationData,
	delta_t: i64,
) -> i64 {
	2000 + (delta_t) * calibration_data.tempsens as i64 / 2_i64.pow(23)
}

pub fn calculate_t2(delta_t: i64) -> i64 {
	delta_t.pow(2) / 2_i64.pow(31)
}
pub fn calculate_off2(actual_temperature: i64) -> i64 {
	5 * (actual_temperature - 2000).pow(2) / 2
}
pub fn calculate_sens2(actual_temperature: i64) -> i64 {
	5 * (actual_temperature - 2000).pow(2) / 2_i64.pow(2)
}

pub fn calculate_off2_sub15(actual_temperature: i64) -> i64 {
	calculate_off2(actual_temperature) + (7 * (actual_temperature + 1500).pow(2))
}
pub fn calculate_sens2_sub15(actual_temperature: i64) -> i64 {
	calculate_sens2(actual_temperature) + (11 * (actual_temperature + 1500).pow(2) / 2)
}

/// Returns (actual temperature, off, sens)
pub fn constant_calculations(
	temperature_value: &u32,
	calibration_data: &CalibrationData,
	delta_t: i64,
) -> (i64, i64, i64) {
	// Calculate a baseline for further compensation
	let mut actual_temperature = calculate_actual_temperature(temperature_value, calibration_data, delta_t);
	let off;
	let sens;
	// Check if compensation needs to be done for sub 20 Celsius
	if actual_temperature < 2000 {
		// Check if compensation needs to be done for sub -15 Celsius
		if actual_temperature < -1500 {
			off = calculate_temperature_offset(calibration_data, delta_t) - calculate_off2_sub15(actual_temperature);
			sens = calculate_temperature_sensitivity(calibration_data, delta_t) - calculate_sens2_sub15(actual_temperature);
		} else {
			off = calculate_temperature_offset(calibration_data, delta_t) - calculate_off2(actual_temperature);
			sens = calculate_temperature_sensitivity(calibration_data, delta_t) - calculate_sens2(actual_temperature);
		}
		actual_temperature = actual_temperature - calculate_t2(delta_t);
	} else {
		// info!("A");
		off = calculate_temperature_offset(calibration_data, delta_t);
		sens = calculate_temperature_sensitivity(calibration_data, delta_t);
	}
	return (actual_temperature, off, sens);
}

pub fn calculate_temperature_offset(
	calibration_data: &CalibrationData,
	delta_t: i64,
) -> i64 {
	((calibration_data.off as i64) * 2_i64.pow(16)) + (calibration_data.tco as i64 * delta_t) / 2_i64.pow(7)
}

pub fn calculate_temperature_sensitivity(
	calibration_data: &CalibrationData,
	delta_t: i64,
) -> i64 {
	((calibration_data.sens as i64) * 2_i64.pow(15)) + (calibration_data.tcs as i64 * delta_t) / 2_i64.pow(8)
}

pub fn calculate_compensated_pressure(
	pressure_value: &u32,
	sens: i64,
	off: i64,
) -> i64 {
	(((pressure_value.clone() as i64 * sens) / 2_i64.pow(21)) - off) / 2_i64.pow(15)
}

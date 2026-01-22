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
	temperature_value: &u32,
	calibration_data: &CalibrationData,
	delta_t: i64,
) -> i64 {
	(((pressure_value.clone() as i64 * calculate_temperature_sensitivity(calibration_data, delta_t)) / 2_i64.pow(21))
		- calculate_temperature_offset(calibration_data, delta_t))
		/ 2_i64.pow(15)
}

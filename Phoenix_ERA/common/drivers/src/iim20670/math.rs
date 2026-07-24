/// IIM-20670 Math / Conversion Functions
use nalgebra::Vector3;

use crate::iim20670::config::{AccelFsSel, GyroFsSel};

// Raw Data Container
/// Complete raw IMU sample (16-bit signed, 2's complement from registers)
#[derive(Clone, Copy, Default, Debug)]
pub struct RawImuSample {
	pub gyro: Vector3<i16>,
	pub accel: Vector3<i16>,
	pub accel_lr: Vector3<i16>,
	pub temp1: i16,
	pub temp2: i16,
}

// Converted Data Container
/// Complete converted IMU sample in physical units
#[derive(Clone, Copy, Debug)]
pub struct ImuSample {
	/// Angular rate in degrees per second
	pub gyro_dps: Vector3<f32>,
	/// Acceleration in g (high-resolution)
	pub accel_g: Vector3<f32>,
	/// Acceleration in g (low-resolution)
	pub accel_lr_g: Vector3<f32>,
	/// Temperature from sensor 1 in degrees Celsius
	pub temperature_1_c: f32,
	/// Temperature from sensor 2 in degrees Celsius
	pub temperature_2_c: f32,
}

impl Default for ImuSample {
	fn default() -> Self {
		Self {
			gyro_dps: Vector3::zeros(),
			accel_g: Vector3::zeros(),
			accel_lr_g: Vector3::zeros(),
			temperature_1_c: 0.0,
			temperature_2_c: 0.0,
		}
	}
}

// Gyroscope Conversion (Section 4.7.1)
/// Convert raw gyroscope vector to degrees per second.
/// Formula per axis: ω = gyro_data / 2^15 * FS
pub fn gyro_raw_to_dps(
	raw: &Vector3<i16>,
	fs_sel: GyroFsSel,
) -> Vector3<f32> {
	let scale = fs_sel.full_scale_dps() / 32768.0;
	raw.cast::<f32>() * scale
}

/// Convert gyroscope dps to radians per second.
pub fn dps_to_rads(dps: &Vector3<f32>) -> Vector3<f32> {
	dps * (core::f32::consts::PI / 180.0)
}

// Accelerometer Conversion (Section 4.8.1)
/// Convert raw accelerometer vector (high-resolution) to g
/// Formula per axis: a = accel_data / 2^15 * FS
pub fn accel_raw_to_g(
	raw: &Vector3<i16>,
	fs_sel: AccelFsSel,
) -> Vector3<f32> {
	let scale = fs_sel.full_scale_g() / 32768.0;
	raw.cast::<f32>() * scale
}

/// Convert raw accelerometer vector (low-resolution) to g
/// Formula per axis: a_lr = accel_data_lr / 2^15 * FS_LR
pub fn accel_lr_raw_to_g(
	raw: &Vector3<i16>,
	fs_sel: AccelFsSel,
) -> Vector3<f32> {
	let scale = fs_sel.full_scale_lr_g() / 32768.0;
	raw.cast::<f32>() * scale
}

/// Convert acceleration from g to m/s².
pub fn g_to_ms2(g: &Vector3<f32>) -> Vector3<f32> {
	g * 9.80665
}

// Temperature Conversion (Section 4.9)
/// Convert a raw temperature register value to degrees Celsius.
/// Formula: TEMPERATURE (°C) = 25 + temp_data / 20
pub fn temp_raw_to_celsius(raw: i16) -> f32 {
	25.0 + (raw as f32 / 20.0)
}

// Full Sample Conversion
/// Convert a complete raw IMU sample to physical units.
pub fn convert_raw_sample(
	raw: &RawImuSample,
	gyro_fs: GyroFsSel,
	accel_fs: AccelFsSel,
) -> ImuSample {
	ImuSample {
		gyro_dps: gyro_raw_to_dps(&raw.gyro, gyro_fs),
		accel_g: accel_raw_to_g(&raw.accel, accel_fs),
		accel_lr_g: accel_lr_raw_to_g(&raw.accel_lr, accel_fs),
		temperature_1_c: temp_raw_to_celsius(raw.temp1),
		temperature_2_c: temp_raw_to_celsius(raw.temp2),
	}
}

use libsql::{params, Result, Transaction};
use messages_prost::iim20670::Imu;

pub async fn save_imu(
	transaction: &Transaction,
	imu: &Imu,
) -> Result<i64> {
	if let Some(data) = &imu.data {
		transaction
			.execute(
				"INSERT INTO Iim20670Imu (
                    timestamp_us,
                    gyro_x, gyro_y, gyro_z,
                    accel_x, accel_y, accel_z,
                    temperature_1, temperature_2,
                    accel_fs, gyro_fs, spi_status,
                    low_res_accel_x, low_res_accel_y, low_res_accel_z
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
				params![
					data.timestamp_us as i64,
					data.gyroscope.as_ref().map(|v| v.x),
					data.gyroscope.as_ref().map(|v| v.y),
					data.gyroscope.as_ref().map(|v| v.z),
					data.accelerometer.as_ref().map(|v| v.x),
					data.accelerometer.as_ref().map(|v| v.y),
					data.accelerometer.as_ref().map(|v| v.z),
					data.temperature_1,
					data.temperature_2,
					data.accelerometer_fs,
					data.gyroscope_fs,
					data.spi_status,
					data.low_resolution_accelerometer.as_ref().map(|v| v.x),
					data.low_resolution_accelerometer.as_ref().map(|v| v.y),
					data.low_resolution_accelerometer.as_ref().map(|v| v.z),
				],
			)
			.await?;
	} else {
		transaction
			.execute("INSERT INTO Iim20670Imu (timestamp_us) VALUES (?)", params![0])
			.await?;
	}
	Ok(transaction.last_insert_rowid())
}

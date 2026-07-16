use libsql::{params, Result, Transaction};
use messages_prost::sbg::{sbg_data, SbgData};

pub async fn save_sbg(
	transaction: &Transaction,
	sbg: &SbgData,
) -> Result<i64> {
	let data_id: i64 = match sbg.data.as_ref().unwrap() {
		sbg_data::Data::UtcTime(utc_time) => {
			let status = utc_time.status.as_ref().map(|s| format!("{:?}", s)).unwrap_or_default();
			let data = utc_time.data.as_ref();
			transaction
                .execute(
                    "INSERT INTO SbgUtcTime (time_stamp, status, year, month, day, hour, minute, second, nano_second, gps_time_of_week) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    params![
                        utc_time.time_stamp,
                        status,
                        data.map(|d| d.year),
                        data.map(|d| d.month),
                        data.map(|d| d.day),
                        data.map(|d| d.hour),
                        data.map(|d| d.minute),
                        data.map(|d| d.second),
                        data.map(|d| d.nano_second),
                        data.map(|d| d.gps_time_of_week),
                    ],
                )
                .await?;
			transaction.last_insert_rowid()
		}
		sbg_data::Data::Air(air) => {
			let status = air.status.as_ref().map(|s| format!("{:?}", s)).unwrap_or_default();
			let data = air.data.as_ref();
			transaction
                .execute(
                    "INSERT INTO SbgAir (time_stamp, status, pressure_abs, altitude, pressure_diff, true_airspeed, air_temperature) VALUES (?, ?, ?, ?, ?, ?, ?)",
                    params![
                        air.time_stamp,
                        status,
                        data.map(|d| d.pressure_abs),
                        data.map(|d| d.altitude),
                        data.map(|d| d.pressure_diff),
                        data.map(|d| d.true_airspeed),
                        data.map(|d| d.air_temperature),
                    ],
                )
                .await?;
			transaction.last_insert_rowid()
		}
		sbg_data::Data::EkfQuat(ekf_quat) => {
			let status = ekf_quat.status.as_ref().map(|s| format!("{:?}", s)).unwrap_or_default();
			let data = ekf_quat.data.as_ref();
			transaction
                .execute(
                    "INSERT INTO SbgEkfQuat (time_stamp, quaternion_w, quaternion_x, quaternion_y, quaternion_z, euler_std_dev_roll, euler_std_dev_pitch, euler_std_dev_yaw, status) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    params![
                        ekf_quat.time_stamp,
                        data.and_then(|d| d.quaternion).map(|q| q.w),
                        data.and_then(|d| d.quaternion).map(|q| q.x),
                        data.and_then(|d| d.quaternion).map(|q| q.y),
                        data.and_then(|d| d.quaternion).map(|q| q.z),
                        data.and_then(|d| d.euler_std_dev).map(|e| e.x),
                        data.and_then(|d| d.euler_std_dev).map(|e| e.y),
                        data.and_then(|d| d.euler_std_dev).map(|e| e.z),
                        status,
                    ],
                )
                .await?;
			transaction.last_insert_rowid()
		}
		sbg_data::Data::EkfNav(ekf_nav) => {
			let status = ekf_nav.status.as_ref().map(|s| format!("{:?}", s)).unwrap_or_default();
			let vel = ekf_nav.velocity.as_ref();
			let pos = ekf_nav.position.as_ref();
			transaction
                .execute(
                    "INSERT INTO SbgEkfNav (time_stamp, status, velocity_north, velocity_east, velocity_down, velocity_std_dev_north, velocity_std_dev_east, velocity_std_dev_down, position_latitude, position_longitude, position_altitude, position_std_dev_latitude, position_std_dev_longitude, position_std_dev_altitude, undulation) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    params![
                        ekf_nav.time_stamp,
                        status,
                        vel.and_then(|v| v.velocity).map(|v| v.x),
                        vel.and_then(|v| v.velocity).map(|v| v.y),
                        vel.and_then(|v| v.velocity).map(|v| v.z),
                        vel.and_then(|v| v.velocity_std_dev).map(|v| v.x),
                        vel.and_then(|v| v.velocity_std_dev).map(|v| v.y),
                        vel.and_then(|v| v.velocity_std_dev).map(|v| v.z),
                        pos.and_then(|p| p.position).map(|p| p.x),
                        pos.and_then(|p| p.position).map(|p| p.y),
                        pos.and_then(|p| p.position).map(|p| p.z),
                        pos.and_then(|p| p.position_std_dev).map(|p| p.x),
                        pos.and_then(|p| p.position_std_dev).map(|p| p.y),
                        pos.and_then(|p| p.position_std_dev).map(|p| p.z),
                        ekf_nav.undulation,
                    ],
                )
                .await?;
			transaction.last_insert_rowid()
		}
		sbg_data::Data::Imu(imu) => {
			let status = imu.status.as_ref().map(|s| format!("{:?}", s)).unwrap_or_default();
			transaction
                .execute(
                    "INSERT INTO SbgImu (time_stamp, status, accelerometer_x, accelerometer_y, accelerometer_z, gyroscope_x, gyroscope_y, gyroscope_z, delta_velocity_x, delta_velocity_y, delta_velocity_z, delta_angle_x, delta_angle_y, delta_angle_z, temperature) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    params![
                        imu.time_stamp,
                        status,
                        imu.accelerometers.as_ref().and_then(|v| v.accelerometers).map(|v| v.x),
                        imu.accelerometers.as_ref().and_then(|v| v.accelerometers).map(|v| v.y),
                        imu.accelerometers.as_ref().and_then(|v| v.accelerometers).map(|v| v.z),
                        imu.gyroscopes.as_ref().and_then(|v| v.gyroscopes).map(|v| v.x),
                        imu.gyroscopes.as_ref().and_then(|v| v.gyroscopes).map(|v| v.y),
                        imu.gyroscopes.as_ref().and_then(|v| v.gyroscopes).map(|v| v.z),
                        imu.accelerometers.as_ref().and_then(|v| v.delta_velocity).map(|v| v.x),
                        imu.accelerometers.as_ref().and_then(|v| v.delta_velocity).map(|v| v.y),
                        imu.accelerometers.as_ref().and_then(|v| v.delta_velocity).map(|v| v.z),
                        imu.gyroscopes.as_ref().and_then(|v| v.delta_angle).map(|v| v.x),
                        imu.gyroscopes.as_ref().and_then(|v| v.delta_angle).map(|v| v.y),
                        imu.gyroscopes.as_ref().and_then(|v| v.delta_angle).map(|v| v.z),
                        imu.temperature,
                    ],
                )
                .await?;
			transaction.last_insert_rowid()
		}
		sbg_data::Data::GpsVel(gps_vel) => {
			let status = gps_vel.status.as_ref().map(|s| format!("{:?}", s)).unwrap_or_default();
			let data = gps_vel.data.as_ref();
			transaction
                .execute(
                    "INSERT INTO SbgGpsVel (time_stamp, status, velocity_north, velocity_east, velocity_down, velocity_acc_north, velocity_acc_east, velocity_acc_down, course, course_acc, time_of_week) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    params![
                        gps_vel.time_stamp,
                        status,
                        data.and_then(|d| d.velocity).map(|v| v.x),
                        data.and_then(|d| d.velocity).map(|v| v.y),
                        data.and_then(|d| d.velocity).map(|v| v.z),
                        data.and_then(|d| d.velocity_acc).map(|v| v.x),
                        data.and_then(|d| d.velocity_acc).map(|v| v.y),
                        data.and_then(|d| d.velocity_acc).map(|v| v.z),
                        data.map(|d| d.course),
                        data.map(|d| d.course_acc),
                        data.map(|d| d.time_of_week),
                    ],
                )
                .await?;
			transaction.last_insert_rowid()
		}
		sbg_data::Data::GpsPos(gps_pos) => {
			// // Print gps_pos as JSON
			// match serde_json::to_string_pretty(&gps_pos) {
			//     Ok(json) => println!("{}", json),
			//     Err(e) => eprintln!("Failed to serialize gps_pos to JSON: {:?}", e),
			// }

			let status = gps_pos.status.as_ref().map(|s| format!("{:?}", s)).unwrap_or_default();
			let data = gps_pos.data.as_ref();
			transaction
                .execute(
                    "INSERT INTO SbgGpsPos (time_stamp, status, latitude, latitude_accuracy, longitude, longitude_accuracy, altitude, altitude_accuracy, undulation, num_sv_used, base_station_id, differential_age, time_of_week) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    params![
                        gps_pos.time_stamp,
                        status,
                        data.map(|d| d.latitude),
                        data.map(|d| d.latitude_accuracy),
                        data.map(|d| d.longitude),
                        data.map(|d| d.longitude_accuracy),
                        data.map(|d| d.altitude),
                        data.map(|d| d.altitude_accuracy),
                        data.map(|d| d.undulation),
                        data.map(|d| d.num_sv_used),
                        data.map(|d| d.base_station_id),
                        data.map(|d| d.differential_age),
                        data.map(|d| d.time_of_week),
                    ],
                )
                .await?;
			transaction.last_insert_rowid()
		}
	};

	Ok(data_id)
}

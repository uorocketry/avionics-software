/// IIM-20670 6-Axis IMU Driver (Minimal)
use defmt::{error, info, warn};
use embassy_time::Timer;
use nalgebra::Vector3;
use uor_peripherals::spi::peripheral::UORMonoCsSPI;

use crate::iim20670::{
	config::{AccelFsSel, FIXED_VALUE, GyroFsSel, SpiReturnStatus, build_spi_command, parse_spi_response, reg},
	math::{ImuSample, RawImuSample, accel_lr_raw_to_g, accel_raw_to_g, convert_raw_sample, gyro_raw_to_dps, temp_raw_to_celsius},
};

// Error Type
#[derive(Debug, defmt::Format)]
pub enum Iim20670Error {
	/// SPI bus communication failure
	Spi,
	/// CRC mismatch on SPI response
	CrcMismatch,
	/// SPI status bits indicate error (RS = 11)
	StatusError,
	/// Data not ready (RS = 10)
	DataNotReady,
	/// WHO_AM_I register returned unexpected value
	WhoAmIMismatch(u8),
	/// Fixed value register returned unexpected value
	FixedValueMismatch(u16),
}

// Driver Struct
pub struct IIM20670<'a> {
	pub spi_service: UORMonoCsSPI<'a>,
	/// Current gyroscope full-scale setting (default: ±655 dps)
	pub gyro_fs: GyroFsSel,
	/// Current accelerometer full-scale setting (default: ±16.384g)
	pub accel_fs: AccelFsSel,
}

impl<'a> IIM20670<'a> {
	// Construction & Initialization
	pub async fn new(spi_service: UORMonoCsSPI<'a>) -> Self {
		let mut imu = IIM20670 {
			spi_service,
			gyro_fs: GyroFsSel::default(),
			accel_fs: AccelFsSel::default(),
		};

		imu.hard_reset().await;
		// Start-up time for register read/write from power-up is 200 ms
		Timer::after_millis(250).await;

		// Verify chip identity
		match imu.verify_identity().await {
			Ok(_) => info!("IIM-20670 identity verified"),
			Err(e) => error!("IIM-20670 identity check failed: {:?}", e),
		}

		imu
	}

	// Low-Level SPI Communication
	async fn spi_transfer_32(
		&mut self,
		command: u32,
	) -> Result<(SpiReturnStatus, u16), Iim20670Error> {
		let tx_bytes = command.to_be_bytes();
		let mut rx_bytes = [0u8; 4];

		match self.spi_service.transfer::<u8>(&mut rx_bytes, &tx_bytes).await {
			Ok(_) => {}
			Err(_) => {
				warn!("SPI transfer failed for IIM-20670");
				return Err(Iim20670Error::Spi);
			}
		}

		let response = u32::from_be_bytes(rx_bytes);
		let (_offset, status, data, crc_valid) = parse_spi_response(response);

		if !crc_valid {
			warn!("CRC mismatch on IIM-20670 SPI response");
			return Err(Iim20670Error::CrcMismatch);
		}

		match status {
			SpiReturnStatus::Error => Err(Iim20670Error::StatusError),
			_ => Ok((status, data)),
		}
	}

	/// Read a 16-bit register from the currently selected bank (Bank 0)
	async fn read_register(
		&mut self,
		offset: u8,
	) -> Result<u16, Iim20670Error> {
		let cmd = build_spi_command(true, offset, 0x0000);
		let (status, data) = self.spi_transfer_32(cmd).await?;

		match status {
			SpiReturnStatus::Success => Ok(data),
			SpiReturnStatus::InProgress => Err(Iim20670Error::DataNotReady),
			_ => Ok(data),
		}
	}

	/// Write a 16-bit value to a register in the currently selected bank (Bank 0)
	async fn write_register(
		&mut self,
		offset: u8,
		data: u16,
	) -> Result<(), Iim20670Error> {
		let cmd = build_spi_command(false, offset, data);
		let (status, _) = self.spi_transfer_32(cmd).await?;

		match status {
			SpiReturnStatus::Success => Ok(()),
			SpiReturnStatus::Error => Err(Iim20670Error::StatusError),
			_ => Ok(()),
		}
	}

	// Reset
	pub async fn hard_reset(&mut self) {
		// Write bit 2 of register 0x18 in bank 0
		match self.write_register(reg::RESET_CTRL, 0x0004).await {
			Ok(_) => info!("IIM-20670 hard reset requested"),
			Err(_) => warn!("Failed to send hard reset to IIM-20670"),
		}
	}

	// Identity Verification
	pub async fn verify_identity(&mut self) -> Result<(), Iim20670Error> {
		let fixed = self.read_register(reg::FIXED_VALUE).await?;
		if fixed != FIXED_VALUE {
			error!("IIM-20670 fixed value mismatch: expected 0x{:04X}, got 0x{:04X}", FIXED_VALUE, fixed);
			return Err(Iim20670Error::FixedValueMismatch(fixed));
		}

		info!("IIM-20670 identity OK (FIXED=0x{:04X})", fixed);
		Ok(())
	}

	// Raw Sensor Data Reading
	/// Read raw gyroscope data for all three axes.
	pub async fn read_gyro_raw(&mut self) -> Result<Vector3<i16>, Iim20670Error> {
		let x = self.read_register(reg::GYRO_X_DATA).await? as i16;
		let y = self.read_register(reg::GYRO_Y_DATA).await? as i16;
		let z = self.read_register(reg::GYRO_Z_DATA).await? as i16;
		Ok(Vector3::new(x, y, z))
	}

	/// Read raw accelerometer data (high-resolution) for all three axes.
	pub async fn read_accel_raw(&mut self) -> Result<Vector3<i16>, Iim20670Error> {
		let x = self.read_register(reg::ACCEL_X_DATA).await? as i16;
		let y = self.read_register(reg::ACCEL_Y_DATA).await? as i16;
		let z = self.read_register(reg::ACCEL_Z_DATA).await? as i16;
		Ok(Vector3::new(x, y, z))
	}

	/// Read raw accelerometer data (low-resolution) for all three axes.
	pub async fn read_accel_lr_raw(&mut self) -> Result<Vector3<i16>, Iim20670Error> {
		let x = self.read_register(reg::ACCEL_X_DATA_LR).await? as i16;
		let y = self.read_register(reg::ACCEL_Y_DATA_LR).await? as i16;
		let z = self.read_register(reg::ACCEL_Z_DATA_LR).await? as i16;
		Ok(Vector3::new(x, y, z))
	}

	/// Read raw temperature sensor 1 data.
	pub async fn read_temp1_raw(&mut self) -> Result<i16, Iim20670Error> {
		Ok(self.read_register(reg::TEMP1_DATA).await? as i16)
	}

	/// Read raw temperature sensor 2 data.
	pub async fn read_temp2_raw(&mut self) -> Result<i16, Iim20670Error> {
		Ok(self.read_register(reg::TEMP2_DATA).await? as i16)
	}

	/// Read a complete raw IMU sample (all sensors in one call).
	pub async fn read_raw_sample(&mut self) -> Result<RawImuSample, Iim20670Error> {
		let gyro = self.read_gyro_raw().await?;
		let temp1 = self.read_temp1_raw().await?;
		let accel = self.read_accel_raw().await?;
		let temp2 = self.read_temp2_raw().await?;
		let accel_lr = self.read_accel_lr_raw().await?;

		Ok(RawImuSample {
			gyro,
			accel,
			accel_lr,
			temp1,
			temp2,
		})
	}

	// Converted Sensor Data Reading
	/// Read gyroscope data converted to degrees per second.
	pub async fn read_gyro_dps(&mut self) -> Result<Vector3<f32>, Iim20670Error> {
		let raw = self.read_gyro_raw().await?;
		Ok(gyro_raw_to_dps(&raw, self.gyro_fs))
	}

	/// Read accelerometer data (high-resolution) converted to g.
	pub async fn read_accel_g(&mut self) -> Result<Vector3<f32>, Iim20670Error> {
		let raw = self.read_accel_raw().await?;
		Ok(accel_raw_to_g(&raw, self.accel_fs))
	}

	/// Read accelerometer data (low-resolution) converted to g.
	pub async fn read_accel_lr_g(&mut self) -> Result<Vector3<f32>, Iim20670Error> {
		let raw = self.read_accel_lr_raw().await?;
		Ok(accel_lr_raw_to_g(&raw, self.accel_fs))
	}

	/// Read temperature sensor 1 in degrees Celsius.
	pub async fn read_temperature_1(&mut self) -> Result<f32, Iim20670Error> {
		let raw = self.read_temp1_raw().await?;
		Ok(temp_raw_to_celsius(raw))
	}

	/// Read temperature sensor 2 in degrees Celsius.
	pub async fn read_temperature_2(&mut self) -> Result<f32, Iim20670Error> {
		let raw = self.read_temp2_raw().await?;
		Ok(temp_raw_to_celsius(raw))
	}

	/// Read a complete IMU sample with all values converted to physical units.
	pub async fn read_sample(&mut self) -> Result<ImuSample, Iim20670Error> {
		let raw = self.read_raw_sample().await?;
		Ok(convert_raw_sample(&raw, self.gyro_fs, self.accel_fs))
	}
}

use embassy_stm32::pac::common::W;
use embassy_time::Timer;
use uor_peripherals::spi::peripheral::UORMonoCsSPI;
use uor_utils::linear_algebra::vectors::{VectorR3, VectorR3F};

use crate::mmc5983ma::utils::{MMC5983MABandwidth, MMC5983MARegisters};

pub struct MMC5983MA<'a> {
	pub spi_service: UORMonoCsSPI<'a>,
}

impl<'a> MMC5983MA<'a> {
	pub fn new(spi_service: UORMonoCsSPI<'a>) -> Self {
		MMC5983MA { spi_service: spi_service }
	}

	// Read from chip when rw_n high
	pub async fn transfer(
		&mut self,
		register: MMC5983MARegisters,
		write_data: u8,
		rw_n: bool,
	) -> u8 {
		let mut read_buf: [u16; 1] = [0];
		let mut write: u16 = 0;
		// Set address bits
		write = write | ((register as u16) << 8);
		write = write | (write_data as u16);

		if rw_n {
			write = write | 0b1000_0000;
		}

		let _ = self.spi_service.transfer::<u16>(&mut read_buf, &mut [write]).await;
		read_buf[0] as u8
	}

	pub async fn start_measurement(
		&mut self,
		mag: bool,
		temp: bool,
	) {
		let mut write_data = 0;
		if mag {
			write_data = write_data | 0b0000_0001;
		}
		if temp {
			write_data = write_data | 0b0000_0010;
		}

		self.transfer(MMC5983MARegisters::InternalControl0, write_data, false).await;
	}

	pub async fn get_raw_mag_data(&mut self) -> VectorR3 {
		let mut x_val: u32 = 0;
		let mut y_val: u32 = 0;
		let mut z_val: u32 = 0;

		x_val = x_val | ((self.transfer(MMC5983MARegisters::XOut0, 0, true).await as u32) << 10);
		x_val = x_val | ((self.transfer(MMC5983MARegisters::XOut1, 0, true).await as u32) << 2);

		y_val = y_val | ((self.transfer(MMC5983MARegisters::YOut0, 0, true).await as u32) << 10);
		y_val = y_val | ((self.transfer(MMC5983MARegisters::YOut1, 0, true).await as u32) << 2);

		z_val = z_val | ((self.transfer(MMC5983MARegisters::ZOut0, 0, true).await as u32) << 10);
		z_val = z_val | ((self.transfer(MMC5983MARegisters::ZOut1, 0, true).await as u32) << 2);

		// The lower two bits of the X,Y, and Z vector magnitudes
		let xyz_lower = self.transfer(MMC5983MARegisters::XYZOut2, 0, true).await as u32;

		// TODO: GET RID OF THE EVIL MAGIC NUMBERS GRRRR
		x_val = x_val | ((xyz_lower & 0b1100_0000) >> 6);
		y_val = y_val | ((xyz_lower & 0b0011_0000) >> 6);
		z_val = z_val | ((xyz_lower & 0b0000_1100) >> 6);

		VectorR3::new(x_val as i64, y_val as i64, z_val as i64)
	}

	pub async fn get_18bit_mag(&mut self) -> VectorR3F {
		let sensor_vector = VectorR3F::from(self.get_raw_mag_data().await);
		let mut output: VectorR3F = VectorR3F::new(-5.0, -5.0, -5.0)
			+ (sensor_vector * VectorR3F::new((10 / 2_i32.pow(18) as f64), (10 / 2_i32.pow(18) as f64), (10 / 2_i32.pow(18) as f64)));
		output
	}

	pub async fn generate_mag_vector(
		&mut self,
		bandwidth: MMC5983MABandwidth,
	) -> VectorR3F {
		self.transfer(MMC5983MARegisters::InternalControl1, bandwidth.clone() as u8, false);
		self.start_measurement(true, false).await;
		// TODO: GET RID OF MORE EVIL MAGIC NUMBERS GRRRR

		match bandwidth {
			MMC5983MABandwidth::Hz100 => {
				Timer::after_millis(9).await;
			}
			MMC5983MABandwidth::Hz200 => {
				Timer::after_millis(5).await;
			}
			MMC5983MABandwidth::Hz400 => {
				Timer::after_millis(3).await;
			}
			MMC5983MABandwidth::Hz800 => {
				Timer::after_millis(1).await;
			}
		}
		// This all assumes 18 bit mode
		self.get_18bit_mag().await
	}
}

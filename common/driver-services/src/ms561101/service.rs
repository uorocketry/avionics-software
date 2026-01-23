// TODO: Add support for the lower temp ran

use defmt::{info, warn};
use embassy_stm32::{
	Peripheral,
	gpio::{AnyPin, Output, Pin},
	lptim::timer,
	sai::B,
};
use embassy_time::{Delay, Timer};
use peripheral_services::{spi::service::SPIService, with_cs};

use crate::ms561101::{
	config::{CalibrationData, CalibrationMasks, Commands, OSR, SamplingCommands},
	math::{calculate_actual_temperature, calculate_compensated_pressure, calculate_temperature_difference, constant_calculations},
	units::{Pressure, Temperature},
};
pub struct MS561101Service<'a> {
	pub spi_service: SPIService<'a>,
	// TODO: This should live in SPU service for obvious reason lol.
	pub calibration_data: CalibrationData,
}

// Maximum SPI frequency is 20MHz
impl<'a> MS561101Service<'a> {
	pub async fn new(spi_service: SPIService<'a>) -> Self {
		let mut service = MS561101Service {
			spi_service: spi_service,
			calibration_data: CalibrationData::default(),
		};
		service.reset().await;
		// Data sheet: Wait 2.8 ms (max) after reset
		Timer::after_millis(3000).await;
		let calibration_data = service.read_calibration_data().await;
		service.calibration_data = calibration_data;
		service
	}

	pub async fn reset(&mut self) {
		match self.spi_service.write::<u8>(&[Commands::Reset as u8]).await {
			Ok(_) => {}
			Err(_) => {
				warn!("Failed to write reset command to MS561101 (Barometer)");
			}
		}
		info!("Chip reset requested");
	}

	async fn read_sensor_component(
		&mut self,
		conversion_delay: u64,
		command: SamplingCommands,
	) -> u32 {
		let mut read_buffer: [u8; 4] = [0; 4];
		let mut write_buffer: [u8; 4] = [Commands::ADC as u8, 0, 0, 0];
		self.spi_service.write::<u8>(&[command as u8]).await;
		// Must wait the time it takes for sensor conversion
		Timer::after_millis(conversion_delay).await;
		// TODO: Look into swapping the write & read to a transfer cmd
		self.spi_service.transfer::<u8>(&mut read_buffer, &mut write_buffer).await;
		u32::from_be_bytes([0, read_buffer[1], read_buffer[2], read_buffer[3]])
	}

	pub async fn read_pressure_raw(
		&mut self,
		oversampling: &OSR,
	) -> u32 {
		self.read_sensor_component(
			oversampling.clone() as u64,
			match oversampling {
				OSR::OSR256 => SamplingCommands::ConvertD1Osr256,
				OSR::OSR512 => SamplingCommands::ConvertD1Osr512,
				OSR::OSR1024 => SamplingCommands::ConvertD1Osr1024,
				OSR::OSR2048 => SamplingCommands::ConvertD1Osr2048,
				OSR::OSR4096 => SamplingCommands::ConvertD1Osr4096,
			},
		)
		.await
	}

	pub async fn read_temperature_raw(
		&mut self,
		oversampling: &OSR,
	) -> u32 {
		self.read_sensor_component(
			oversampling.clone() as u64,
			match oversampling {
				OSR::OSR256 => SamplingCommands::ConvertD2Osr256,
				OSR::OSR512 => SamplingCommands::ConvertD2Osr512,
				OSR::OSR1024 => SamplingCommands::ConvertD2Osr1024,
				OSR::OSR2048 => SamplingCommands::ConvertD2Osr2048,
				OSR::OSR4096 => SamplingCommands::ConvertD2Osr4096,
			},
		)
		.await
	}

	pub async fn read_temperature(
		&mut self,
		oversampling: &OSR,
	) -> i64 {
		let raw_temperature = self.read_temperature_raw(oversampling).await;
		let delta_t = calculate_temperature_difference(&raw_temperature, &self.calibration_data);
		calculate_actual_temperature(&raw_temperature, &self.calibration_data, delta_t)
	}

	pub async fn read_sample(
		&mut self,
		oversampling: OSR,
	) -> (Temperature, Pressure) {
		let temperature_raw = self.read_temperature_raw(&oversampling).await;
		// let temperature = self.read_temperature(&oversampling).await;
		let pressure_raw = self.read_pressure_raw(&oversampling).await;

		let delta_t = calculate_temperature_difference(&temperature_raw, &self.calibration_data);

		let constants = constant_calculations(&temperature_raw, &self.calibration_data, delta_t);
		let pressure = calculate_compensated_pressure(&pressure_raw, constants.2, constants.1);
		let temperature = constants.0;
		(Temperature::new(temperature), Pressure::new(pressure))
	}

	async fn read_calibration_field(
		&mut self,
		mask: CalibrationMasks,
	) -> u16 {
		let mut buffer = [0; 1];
		with_cs!(self.spi_service, {
			self.spi_service.write_nocs::<u8>(&[Commands::PROM as u8 | mask as u8]).await;
			self.spi_service.read_nocs::<u16>(&mut buffer).await;
		});

		buffer[0]
	}

	async fn read_calibration_data(&mut self) -> CalibrationData {
		let mut calb_data = CalibrationData::default();

		calb_data.sens = self.read_calibration_field(CalibrationMasks::SENS).await;
		calb_data.off = self.read_calibration_field(CalibrationMasks::OFF).await;
		calb_data.tcs = self.read_calibration_field(CalibrationMasks::TCS).await;
		calb_data.tco = self.read_calibration_field(CalibrationMasks::TCO).await;
		calb_data.tref = self.read_calibration_field(CalibrationMasks::TREF).await;
		calb_data.tempsens = self.read_calibration_field(CalibrationMasks::TEMPSENS).await;

		info!("READ THE FOLLOWING CONFIGURATION DATA: ");
		info!("\tSENS: {}", calb_data.sens);
		info!("\tOFF: {}", calb_data.off);
		info!("\tTPS: {}", calb_data.tcs);
		info!("\tTCO: {}", calb_data.tco);
		info!("\tTREF: {}", calb_data.tref);
		info!("\tTEMPSENS: {}", calb_data.tempsens);

		calb_data
	}
}

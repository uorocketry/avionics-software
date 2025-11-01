use core::fmt::Debug;
use core::str::FromStr;

use defmt::Format;
use heapless::{format, String, Vec};
use strum::EnumCount;

use crate::adc::types::AdcDevice;
use crate::linear_transformation::types::LinearTransformation;
use crate::pressure::config::MAX_CALIBRATION_DATA_POINTS;
use crate::pressure::service::PressureService;
use crate::pressure::types::{PressureChannel, PressureServiceError};

// Calibration logic has been separated into its own file for clarity
impl<const ADC_COUNT: usize> PressureService<ADC_COUNT> {
	pub async fn calibrate(&mut self) -> Result<(), PressureServiceError> {
		// Prompt for ADC index
		let adc_index: usize = self.prompt("Starting pressure calibration. Enter ADC index (Starts from 0):\"\n").await?;
		if adc_index >= AdcDevice::COUNT {
			self.send_message("Invalid ADC index.\n").await?;
			return Ok(());
		}
		let adc = AdcDevice::from(adc_index);

		// Prompt for channel
		let channel_index: usize = self.prompt("Enter pressure channel index (Starts from 0):\"\n").await?;
		if channel_index >= PressureChannel::COUNT {
			self.send_message("Invalid channel index.\n").await?;
			return Ok(());
		}
		let channel = PressureChannel::from(channel_index);

		{
			let message: String<64> =
				format!("Calibrating ADC {}, Channel {}\n", adc_index, channel_index).map_err(|_| PressureServiceError::FormatError)?;
			self.send_message(message.as_str()).await?;
		}

		// Prompt for number of data points
		let data_points_count: u8 = self.prompt("Enter number of data points to use for the linear fit:\n").await?;
		if data_points_count < 2 {
			self.send_message("Minimum 2 data points is required.\n").await?;
			return Ok(());
		}
		if data_points_count > MAX_CALIBRATION_DATA_POINTS as u8 {
			let error_message: String<64> =
				format!("Too many data points. Maximum is {}.\n", MAX_CALIBRATION_DATA_POINTS).map_err(|_| PressureServiceError::FormatError)?;
			self.send_message(error_message.as_str()).await?;
			return Ok(());
		}

		// Deregister any existing transformation for this channel
		self.linear_transformation_service.deregister_transformation(adc, channel);

		// Start collecting data points
		let mut calibration_data_points: Vec<CalibrationDataPoint, MAX_CALIBRATION_DATA_POINTS> = Vec::new();
		for data_point_index in 0..data_points_count {
			let message: String<64> =
				format!("Data Point #{}. Enter expected value:\n", data_point_index + 1).map_err(|_| PressureServiceError::FormatError)?;
			let expected_pressure: f64 = self.prompt(message.as_str()).await?;
			let measured_pressure: f64 = self.read_pressure(adc, channel).await?.pressure;
			let data_point = CalibrationDataPoint {
				expected_pressure,
				measured_pressure,
			};
			calibration_data_points.push(data_point).unwrap(); // Safe due to prior checks

			let confirmation_message: String<64> = format!("Expected = {:.2}, Measured = {:.2}\n", expected_pressure, measured_pressure)
				.map_err(|_| PressureServiceError::FormatError)?;
			self.send_message(confirmation_message.as_str()).await?;
		}

		// Perform Ordinary Least Squares Fit
		let transformation = self.run_ordinary_least_squares_fit(adc, channel, calibration_data_points);
		let result_message: String<128> = format!(
			"Ordinary Least Squares Fit complete. Scale: {:.6}, Offset: {:.2} °C\n",
			transformation.scale, transformation.offset
		)
		.map_err(|_| PressureServiceError::FormatError)?;
		self.send_message(result_message.as_str()).await?;

		// Update calibration for the channel
		self.linear_transformation_service.save_transformation(transformation).await?;
		Ok(())
	}

	fn run_ordinary_least_squares_fit(
		&self,
		adc: AdcDevice,
		channel: PressureChannel,
		data_points: Vec<CalibrationDataPoint, MAX_CALIBRATION_DATA_POINTS>,
	) -> LinearTransformation<PressureChannel, f64> {
		// Keeping all types as f64
		let data_points_count: f64 = data_points.len() as f64;

		// Accumulate sums
		let mut sum_x: f64 = 0.0; // measured
		let mut sum_y: f64 = 0.0; // expected
		let mut sum_xx: f64 = 0.0; // measured^2
		let mut sum_xy: f64 = 0.0; // measured * expected

		for data_point in data_points.iter() {
			let x = data_point.measured_pressure;
			let y = data_point.expected_pressure;
			sum_x += x;
			sum_y += y;
			sum_xx += x * x;
			sum_xy += x * y;
		}

		// Denominator for slope (variance term)
		let denom = data_points_count * sum_xx - sum_x * sum_x;
		let scale = (data_points_count * sum_xy - sum_x * sum_y) / denom;
		let offset = (sum_y - scale * sum_x) / data_points_count;

		LinearTransformation { adc, channel, scale, offset }
	}

	async fn prompt<T>(
		&mut self,
		prompt: &str,
	) -> Result<T, PressureServiceError>
	where
		T: FromStr,
		<T as FromStr>::Err: core::fmt::Debug, {
		let mut serial_service = self.serial_service.lock().await;
		let mut input = String::<256>::new();
		serial_service.write_str(prompt).await?;
		serial_service.read_line(&mut input).await?;
		Ok(input.trim().parse().map_err(|_| PressureServiceError::FormatError)?)
	}

	async fn send_message(
		&mut self,
		message: &str,
	) -> Result<(), PressureServiceError> {
		let mut serial_service = self.serial_service.lock().await;
		serial_service.write_str(message).await?;
		Ok(())
	}
}

// Represents a single calibration data point used to perform a linear regression
#[derive(Debug, Clone, Copy, Format)]
pub struct CalibrationDataPoint {
	// Expected pressure value in degrees Celsius measured by the calibration instrument
	pub expected_pressure: f64,

	// Value measured by the ADC
	pub measured_pressure: f64,
}

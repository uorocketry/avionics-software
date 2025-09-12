pub mod registers;
pub mod commands;
pub mod parameters;

use defmt::warn;
use embassy_time::Timer;
use embedded_hal::{digital::{InputPin, OutputPin}, spi::Operation};
use embedded_hal_async::spi::SpiDevice;

use registers::Register;
use commands::Command;
use parameters::{ChannelShift, Gain, Filter, DataRate, ReferenceVoltageSource, AnalogChannel, MAX_SIGNED_CODE_SIZE};

pub struct Ads1262<SPI, DataReady, Reset, Start> {
	spi_device: SPI,
	data_ready: DataReady,
	reset: Reset,
	start: Start,
	channel_shift: ChannelShift,
	enable_internal_reference_voltage: bool,
	reference_voltage_source: ReferenceVoltageSource,
	gain: Gain,
	filter: Filter,
	data_rate: DataRate,
}

impl<SPI, E, DataReady, Reset, Start> Ads1262<SPI, DataReady, Reset, Start>
where
	SPI: SpiDevice<Error=E>,
	DataReady: InputPin,
	Reset: OutputPin,
	Start: OutputPin,
{
	pub fn new(
		spi_device: SPI,
		data_ready: DataReady,
		reset: Reset,
		start: Start,
		reference_voltage_source: ReferenceVoltageSource,
		channel_shift: ChannelShift,
		enable_internal_reference_voltage: bool,
		gain: Gain,
		filter: Filter,
		data_rate: DataRate
	) -> Self {
		Self {
			spi_device,
			data_ready,
			reset,
			start,
			reference_voltage_source,
			channel_shift,
			enable_internal_reference_voltage,
			gain,
			filter,
			data_rate
		}
	}

	async fn read_single_ended(
		&mut self,
		channel: AnalogChannel,
	) -> Result<f32, E> {
		self.set_channels(channel, AnalogChannel::AINCOM).await?;
		self.wait_for_next_data().await;
		let code = self.read_data_code().await?;
		Ok(self.convert_code_to_volts(code))
	}

	async fn read_differential(
		&mut self,
		positive: AnalogChannel,
		negative: AnalogChannel
	) -> Result<f32, E> {
		self.set_channels(positive, negative).await?;
		self.wait_for_next_data().await;
		let code = self.read_data_code().await?;
		Ok(self.convert_code_to_volts(code))
	}

	pub async fn reset_hardware(&mut self) -> Result<(), E> {
		self.reset.set_low().ok();
		Timer::after_millis(2).await;
		self.reset.set_high().ok();
		Timer::after_millis(5).await;
		Ok(())
	}

	async fn send_command(&mut self, command: Command) -> Result<(), E> {
		self.spi_device.write(&[command as u8]).await?;
		Ok(())
	}

	async fn set_channels(
		&mut self,
		positive: AnalogChannel,
		negative: AnalogChannel
	) -> Result<(), E> {
		// Shift positive channel to the left by 4 bits and combine with negative channel using bitwise OR
		// | dddd | dddd |
		// | AINP | AINN |

		self.write_register(
			Register::INPMUX,
			((positive as u8) << 4) | (negative as u8)
		).await
	}

	async fn read_data_code(&mut self) -> Result<i32, E> {
		// Send the RDATA1 command followed by 4 dummy bytes to read the 32-bit result 4 * 8 = 32 bits
		let tx = [ Command::RDATA1 as u8, 0, 0, 0, 0];

		// Receiving buffer is 5 bytes: first byte is a dummy byte for the command, next four are the 32-bit result
		let mut rx = [0u8; 5];

		self.spi_device.transaction(&mut [
			Operation::Write(&tx),
			Operation::Read(&mut rx)
		]).await?;

		// Skip the first part because spi sends a byte for every byte you send it since it's duplex and we're using transaction
		let b = &rx[1..5];

		// Convert the 4 bytes to a signed 32-bit integer
		let code = i32::from_be_bytes([b[0], b[1], b[2], b[3]]);
		Ok(code)
	}

	pub fn convert_code_to_volts(&self, code: i32) -> f32 {
		// Convert a 32‑bit two’s‑complement code to volts, using current VREF and PGA gain.
		let full_scale_range: f32 = self.reference_voltage_source.to_volts() / (self.gain as u8 as f32);
		(code as f64 / MAX_SIGNED_CODE_SIZE) as f32 * full_scale_range
	}

	async fn write_register(&mut self, register: Register, value: u8) -> Result<(), E> {
		// Mask to 5 bits just in case, to remove the leading bits
		let mut address = register as u8;
		address = address & 0x1F;

		// Add the write register opcode prefix 010rrrrr (40h+000rrrrr)
		let op1 = 0x40 | address;

		// Number of registers to write - 1 (we're writing one register, so this is 0)
		let op2 = 0x00;

		let tx = [op1, op2, value];
		self.spi_device.write(&tx).await?;
		Ok(())
	}

	async fn read_register(&mut self, register: Register) -> Result<u8, E> {
		let mut address = register as u8;
		// Mask to 5 bits just in case, to remove the leading bits
		address = address & 0x1F;

		// Add the read register opcode prefix 001rrrrr (20h+000rrrrr)
		let op1 = 0x20 | address;

		// Number of registers to read - 1 (we're reading one register, so this is 0)
		let op2 = 0x00;

		// Receiving buffer is 3 bytes: first two are dummy bytes for the opcodes, third is the register value
		let mut rx = [0u8; 3];
		let tx = [op1, op2, 0x00];
		self.spi_device.transaction(&mut [
			Operation::Write(&tx),
			Operation::Read(&mut rx)
		]).await?;

		// Skip the first two bytes because spi sends a byte for every byte you send it since it's duplex and we're using transaction
		Ok(rx[2])
	}

	async fn wait_for_next_data(&mut self) {
		loop {
			if self.data_ready.is_low().unwrap_or(false) {
				break;
			}
			Timer::after_micros(5).await;
		}
	}

	async fn ensure_configured(&mut self) -> Result<(), E> {
		self.send_command(Command::STOP1).await?;

		self.ensure_reference_voltage_source_configured().await?;
		self.ensure_internal_reference_voltage_and_channel_shift_configured().await?;

		// Disable all interface options (status byte, CRC, watchdog)
		// SHOULDDO: make these configurable
		self.write_register(Register::INTERFACE, 0x00).await?;

		// Clears MODE0 so chop mode is disabled, conversion delay is zero, run mode is continuous.
		// SHOULDDO: make these configurable
		self.write_register(Register::MODE0, 0x00).await?;

		self.ensure_offset_calibration_configured().await?;
		self.ensure_full_scale_calibration_configured().await?;
		self.ensure_filter_configured().await?;
		self.ensure_gain_and_data_rate_configured().await?;

		// Short the channels together before we begin
		self.set_channels(AnalogChannel::AINCOM, AnalogChannel::AINCOM).await?;
		self.send_command(Command::START1).await?;
		Ok(())
	}

	async fn ensure_reference_voltage_source_configured(&mut self) -> Result<(), E> {
		let mut register_value: u8 = 0x00;

		match self.reference_voltage_source {
			ReferenceVoltageSource::Avdd => {
				register_value |= 0b100 << 3; // AVDD
				register_value |= 0b100; // AVSS
			},
			ReferenceVoltageSource::Internal2_5 => {
				register_value |= 0b000 << 3; // INTERNAL 2.5V
				register_value |= 0b100; // AVSS
			},
		}

		self.write_register(Register::REFMUX, register_value).await?;
		Ok(())
	}

	async fn ensure_internal_reference_voltage_and_channel_shift_configured(&mut self) -> Result<(), E> {
		let mut register_value: u8 = 0x00;

		if self.enable_internal_reference_voltage {
			register_value |= 1 << 0;
		}

		if matches!(self.channel_shift, ChannelShift::MidSupply) {
			if matches!(self.reference_voltage_source, ReferenceVoltageSource::Internal2_5) {
				warn!("Channel shift is set to mid-supply while ADC reference is internal 2.5V. This leads to the zero point being at ADC max.");
			}
			register_value |= 1 << 1;
		}

		self.write_register(Register::POWER, register_value).await?;
		Ok(())
	}

	async fn ensure_filter_configured(&mut self) -> Result<(), E> {
		let mut register_value: u8 = 0x0;
		register_value |= (self.filter as u8) << 5;
		self.write_register(Register::MODE1, register_value).await
	}

	async fn ensure_gain_and_data_rate_configured(&mut self) -> Result<(), E> {
		let mut register_value: u8 = 0x0;
		register_value |= (self.gain as u8) << 4;
		register_value |= self.data_rate as u8;
		register_value &= 0b0111_1111; // Ensure bypass bit (bit 7) is 0
		self.write_register(Register::MODE2, register_value).await?;
		Ok(())
	}

	async fn ensure_offset_calibration_configured(&mut self) -> Result<(), E> {
		// SHOULD DO: implement
		self.write_register(Register::OFCAL0, 0x00).await?;
		self.write_register(Register::OFCAL1, 0x00).await?;
		self.write_register(Register::OFCAL2, 0x00).await?;
		Ok(())
	}

	async fn ensure_full_scale_calibration_configured(&mut self) -> Result<(), E> {
		// SHOULD DO: implement
		Ok(())
	}
}
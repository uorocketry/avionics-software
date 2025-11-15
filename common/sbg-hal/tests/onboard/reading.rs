#![feature(impl_trait_in_assoc_type)]
#![no_std]
#![no_main]

// Define the CRC function outside the test module
fn crc_function(data: &[u8]) -> u16 {
	let poly: u16 = 0x8408;
	let mut crc: u16 = 0x0000;
	for &byte in data {
		crc ^= byte as u16;
		for _ in 0..8 {
			if crc & 1 != 0 {
				crc = (crc >> 1) ^ poly;
			} else {
				crc >>= 1;
			}
		}
	}
	(crc >> 8) | (crc << 8)
}

#[cfg(test)]
#[embedded_test::tests]
mod tests {
	use bytemuck::{checked::from_bytes, try_from_bytes};
	use defmt::error;
	use defmt_rtt as _;
	use embassy_stm32::{
		Config, bind_interrupts, crc, peripherals,
		usart::{self, BufferedUart},
	};
	use embedded_io_async::{Read, Write};
	use sbg_hal::{data_structs::messages::SbgEcomLogMag, sbg_device::SbgDevice, sbg_frame::FrameTypes};

	use super::crc_function;

	bind_interrupts!(struct Irqs {
		UART4 => usart::BufferedInterruptHandler<peripherals::UART4>;
	});

	// Define the CRC function type
	type CrcFn = fn(&[u8]) -> u16;

	// Static buffers to avoid lifetime issues
	static mut TX_BUFFER: [u8; 512] = [0; 512];
	static mut RX_BUFFER: [u8; 512] = [0; 512];
	static mut SBG_BUFFER: [u8; 4096] = [0; 4096];

	struct TestContext {
		uart: BufferedUart<'static>,
	}

	#[init]
	async fn init() -> TestContext {
		let config = Config::default();
		let device = embassy_stm32::init(config);

		// Need to test the CRC hardware accelerator
		// let crc_config = crc::Config::new(0, 0, 0, 0, 0);

		// let crc_accelerator = Crc::new(device.CRC, crc_config);

		let (tx_buf, rx_buf) = unsafe { (&mut *core::ptr::addr_of_mut!(TX_BUFFER), &mut *core::ptr::addr_of_mut!(RX_BUFFER)) };

		let mut uart_line = BufferedUart::new(device.UART4, Irqs, device.PA1, device.PA0, tx_buf, rx_buf, usart::Config::default())
			.expect("Failed to create UART instance");
		TestContext { uart: uart_line }
	}

	#[test]
	async fn await_log_mag(mut ctx: TestContext) {
		error!("Test starting...");

		let sbg_buffer = unsafe { &mut *core::ptr::addr_of_mut!(SBG_BUFFER) };

		let mut sbg = SbgDevice::new(&mut ctx.uart, crc_function as CrcFn, sbg_buffer);

		loop {
			let frame: FrameTypes;
			match sbg
				.read_frame_by_msgid(sbg_hal::data_structs::frame_identifiers::MESSAGE::SBG_ECOM_LOG_MAG)
				.await
			{
				Ok(x) => {
					frame = x;
				}
				Err(_) => {
					// error!("Frame not found");
					continue;
				}
			}
			// match sbg.read_frame().await {
			// 	Ok(x) => {
			// 		frame = x;
			// 	}
			// 	Err(_) => {
			// 		error!("Frame not found");
			// 		continue;
			// 	}
			// }
			let data_struct: SbgEcomLogMag;
			if frame.get_msgid() == 4 {
				// NOTE THAT INDEX 0 TO THE LENGTH OF THE MESSAGE MUST BE SPECIFIED BY SLICE. IF NOT, BYTEMUCK WILL ACT AS IF THERE IS 4096 BYTES IN STRUCT
				data_struct = *from_bytes::<SbgEcomLogMag>(&frame.get_data()[0..30]);

				let accel_x = data_struct.accel_x;
				let accel_y = data_struct.accel_y;
				let accel_z = data_struct.accel_z;

				error!("Accel x - {}", accel_x);
				error!("Accel y - {}", accel_y);
				error!("Accel z - {}", accel_z);
			}
		}
	}
}

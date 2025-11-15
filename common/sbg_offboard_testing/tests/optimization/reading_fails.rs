//! Used to test the "cleanliness" of reading fails (no frame found, crc fail, etc)

use colored::Colorize;
use devtimer::DevTime;
use sbg_hal::data_structs::frame_identifiers::MESSAGE;
use sbg_hal::sbg_device::{FrameError, SbgDevice};
use sbg_hal::sbg_frame::FrameTypes;
use sbg_hal::sbg_test::sbg_tester::SbgTester;
use tokio;

#[tokio::test]
async fn test_read_bad_crc() {
	println!("{}", "\n\nStarting algorithim with bad crc frame".yellow());

	// Creates a valid known dataframe (recieved from logic analyzer)
	let mut sample_frame_chunk_1: [u8; 39] = [
		0xFF, 0x5A, 0x04, 0x00, 0x1E, 0x00, 0xE8, 0x11, 0x92, 0x0A, 0xFF, 0x01, 0x40, 0x2F, 0x90, 0x3E, 0x80, 0xE2, 0x85, 0xBD, 0x65, 0x1E, 0x9C,
		0xBE, 0x84, 0xAD, 0xC4, 0xBE, 0x3C, 0x10, 0x1D, 0xC1, 0x25, 0xB7, 0x07, 0x3F, 0xE4, 0x91, 0x33,
	];
	let mut sample_frame_chunk_2: [u8; 0] = [];
	let mut tester = SbgTester::new(&sample_frame_chunk_1, &sample_frame_chunk_2);

	let crc_fn = |data: &[u8]| -> u16 {
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
	};

	// Creates the internal buffer used by the SBG struct to handle all data recieved
	let mut internal_buffer: [u8; 4096] = [0; 4096];
	let mut SbgDevice = SbgDevice::new(&mut tester, crc_fn, &mut internal_buffer);

	let frame = SbgDevice.read_frame().await;

	let mut error_result = FrameError::NoFrameFound;
	match frame {
		Ok(_) => {
			println!("{}", "DID NOT FAIL".red());
		}
		Err(x) => {
			error_result = x;
		}
	}
	if error_result == FrameError::CRCBad {
		println!("{}", "FAILED SUCCESSFULLY".green());
	} else {
		println!("{}", "INCORRECT ERROR CODE".red());
	}
	assert!(error_result == FrameError::CRCBad)
}

#[tokio::test]
async fn test_read_no_frame() {
	println!("{}", "\n\nStarting algorithim with undesired frame".yellow());

	// Creates a valid known dataframe (recieved from logic analyzer)
	let mut sample_frame_chunk_1: [u8; 39] = [
		0xFF, 0x5A, 0x02, 0x00, 0x1E, 0x00, 0xE8, 0x11, 0x92, 0x0A, 0xFF, 0x01, 0x40, 0x2F, 0x90, 0x3E, 0x80, 0xE2, 0x85, 0xBD, 0x65, 0x1E, 0x9C,
		0xBE, 0x84, 0xAD, 0xC4, 0xBE, 0x3C, 0x10, 0x1D, 0xC1, 0x25, 0xB7, 0x07, 0x3F, 0xE4, 0x81, 0x33,
	];
	let mut sample_frame_chunk_2: [u8; 0] = [];
	let mut tester = SbgTester::new(&sample_frame_chunk_1, &sample_frame_chunk_2);

	let crc_fn = |data: &[u8]| -> u16 {
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
	};

	// Creates the internal buffer used by the SBG struct to handle all data recieved
	let mut internal_buffer: [u8; 4096] = [0; 4096];
	let mut SbgDevice = SbgDevice::new(&mut tester, crc_fn, &mut internal_buffer);

	let frame = SbgDevice.read_frame_by_msgid(MESSAGE::SBG_ECOM_LOG_STATUS).await;

	let mut error_result = FrameError::CRCBad;
	match frame {
		Ok(_) => {
			println!("{}", "DID NOT FAIL".red());
		}
		Err(x) => {
			error_result = x;
		}
	}
	if error_result == FrameError::NoFrameFound {
		println!("{}", "FAILED SUCCESSFULLY".green());
	} else {
		println!("{}", "INCORRECT ERROR CODE".red());
	}
	assert!(error_result == FrameError::NoFrameFound)
}

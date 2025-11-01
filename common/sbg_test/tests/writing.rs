use tokio;
use sbg_test::sbg_tester::SbgTester;
use sbg_hal::sbg_device::SbgDevice;
use sbg_hal::data_structs::commands::*;

// Tests reading of SbgEcomLogMag
#[tokio::test] 
async fn test_write_01 (){
	// Creates a valid known dataframe (recieved from logic analyzer)
    let mut sample_frame_chunk_1:[u8; 1] = [0x00];
	let mut sample_frame_chunk_2: [u8; 1] = [0x00];
    let expected_output = [5, 16, 28, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
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
	let mut internal_buffer:[u8; 256] = [0; 256];
	let mut SbgDevice = SbgDevice::new(tester, crc_fn, &mut internal_buffer);

    let command = SbgEcomCmdInitParameters{ init_lad: 0.0,
    init_long: 0.0,
    init_alt: 0.0,
    year: 2025,
    month: 10,
    day: 25
};
    let crc = crc_fn(&expected_output);
    SbgDevice.write_frame(command).await;
	assert!(crc == 0x2002);

}
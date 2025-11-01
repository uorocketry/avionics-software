use tokio;
use sbg_test::sbg_tester::SbgTester;
use sbg_hal::sbg_device::SbgDevice;

// Tests reading of SbgEcomLogMag
#[tokio::test] 
async fn test_read_01 (){
	// Creates a valid known dataframe (recieved from logic analyzer)
    let mut sample_frame_chunk_1:[u8; 10] = [0xFF, 0x5A, 0x04, 0x00, 0x1E, 0x00, 0xE8, 0x11, 0x92, 0x0A];
	let mut sample_frame_chunk_2: [u8; 29] = [0xFF, 0x01, 0x40, 0x2F, 0x90, 0x3E, 0x80, 0xE2, 0x85, 0xBD, 0x65, 0x1E, 0x9C, 0xBE, 0x84, 0xAD, 0xC4, 0xBE, 0x3C, 0x10, 0x1D, 0xC1, 0x25, 0xB7, 0x07, 0x3F, 0xE4, 0x99, 0x33];
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

	let frame = SbgDevice.read_frame().await;

	assert!(frame.get_crc() == 0xE499)

}
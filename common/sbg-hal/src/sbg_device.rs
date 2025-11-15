use bytemuck::NoUninit;
use defmt::error;
// use defmt::warn;
// use defmt::{error, warn};
use embedded_io_async::{Read, Write};

use crate::constants::{CLASS_FIELD_OFFSET, END_BIT_OFFSET, LENGTH_OFFSET_HIGH, LENGTH_OFFSET_LOW, MESSAGE_FIELD_OFFSET, PRE_DATA_OFFSET_TRUE};
use crate::data_structs::commands::SbgCommand;
use crate::data_structs::frame_identifiers::MESSAGE;
use crate::data_structs::*;
use crate::sbg_frame::{FrameTypes, SbgFrameFactory, SbgFrameStandard};

pub const BUFFER_SIZE: usize = 4096;
pub const SYNC_BIT_1: u8 = 0xFF;
pub const SYNC_BIT_2: u8 = 0x5A;
pub const END_BIT: u8 = 0x33;

pub const PRE_DATA_OFFSET_CRC: usize = 4;

pub const BAIL_FROM_LOOP: usize = 100;

pub struct SbgDevice<'a, U, T> {
	datastream_provider: &'a mut U,
	crc_provider: T,
	pub buffer: &'a mut [u8],
	buffer_index: usize,
	buffer_max: usize,
}

#[derive(Debug)]
pub enum SeekError {
	FailedToFindBytes,
}

#[derive(Debug, PartialEq)]

pub enum FrameError {
	NoFrameFound,
	CRCBad,
}

fn generate_owned_buffer(buffer: &[u8]) -> [u8; 4086] {
	let mut owned = [0; 4086];
	for i in 0..buffer.len() {
		owned[i] = buffer[i]
	}
	owned
}

fn seek_bytes(
	bytes_to_find: &[u8],
	buffer: &[u8],
) -> Result<usize, SeekError> {
	let mut tally_of_found = 0;
	let mut index_of_bit: isize = 0;
	let mut bit_tgt = bytes_to_find[0];
	// panic!("BIFFER {:?}", buffer)

	for i in 0..buffer.len() {
		if buffer[i] == bit_tgt {
			tally_of_found += 1;

			if tally_of_found == bytes_to_find.len() {
				index_of_bit = i as isize - bytes_to_find.len() as isize + 1;
				// panic!("Index {}", index_of_bit as usize);

				return Ok(index_of_bit as usize);
			}

			bit_tgt = bytes_to_find[tally_of_found];
		} else {
			tally_of_found = 0;
			bit_tgt = bytes_to_find[tally_of_found];
		}
	}
	return Err(SeekError::FailedToFindBytes);
}

impl<'a, U, T> SbgDevice<'a, U, T>
where
	U: Read + Write,
	T: Fn(&[u8]) -> u16,
{
	pub fn new(
		datastream_provider: &'a mut U,
		crc_provider: T,
		buffer: &'a mut [u8],
	) -> SbgDevice<'a, U, T> {
		// recieves the max size of the buffer here as a mutable borrow occurs within struct creation (mutable borrow is necissary as the buffer size is not known)
		let max = buffer.len();
		SbgDevice {
			datastream_provider,
			crc_provider: crc_provider,
			buffer: buffer,
			buffer_index: 0,
			buffer_max: max,
		}
	}

	fn append_to_internal_buffer(
		&mut self,
		data_to_append: &[u8],
	) {
		for i in data_to_append {
			// Checks if buffer is overflowing and resets to 0
			if self.buffer_index >= self.buffer_max - 1 {
				//
				self.buffer_index = 0;
			}
			self.buffer[self.buffer_index] = i.clone();
			self.buffer_index += 1;
		}
	}

	// Checks the provided frame's CRC for data integrity
	fn check_crc(
		&self,
		frame: &FrameTypes,
	) -> bool {
		let mut buffer = [0; BUFFER_SIZE];
		let frame_data = frame.get_data();
		let trimmed_data: &[u8];

		// Appends frame's message id, class, and length. See: <https://developer.sbg-systems.com/sbgECom/5.3/md_doc_2binary_protocol.html#crcDefinition>
		buffer[0] = frame.get_msgid();
		buffer[1] = frame.get_class() as u8;
		buffer[2] = (frame.get_length()) as u8;
		buffer[3] = (frame.get_length() >> 8) as u8;

		for i in 0..frame_data.len() {
			buffer[PRE_DATA_OFFSET_CRC + i] = frame_data[i];
		}

		// Trim out placeholder 0s for CRC
		trimmed_data = &buffer[0..(frame.get_length() as usize + PRE_DATA_OFFSET_CRC)];

		(self.crc_provider)(trimmed_data) == frame.get_crc()
	}

	// Put this in check crc function
	fn generate_crc(
		&self,
		msg: u8,
		class: u8,
		length: u16,
		data: &[u8],
	) -> u16 {
		let mut buffer = [0; BUFFER_SIZE];
		let trimmed_data: &[u8];

		// Appends frame's message id, class, and length. See: <https://developer.sbg-systems.com/sbgECom/5.3/md_doc_2binary_protocol.html#crcDefinition>
		buffer[0] = msg;
		buffer[1] = class;
		buffer[2] = length as u8;
		buffer[3] = (length >> 8) as u8;

		for i in 0..data.len() {
			buffer[PRE_DATA_OFFSET_CRC + i] = data[i];
		}

		// Trim out placeholder 0s for CRC
		trimmed_data = &buffer[0..(length as usize + PRE_DATA_OFFSET_CRC)];

		(self.crc_provider)(trimmed_data)
	}

	pub async fn read_frame(&mut self) -> Result<FrameTypes, FrameError> {
		self.buffer_index = 0;

		// Initializes a buffer for the read data. Locates sync bits 0xFF and 0x5A
		let mut init_found = false;
		let mut frame_captured = false;

		let mut difference_error = 0;
		// Searches for the start and end of the package
		let mut packet_start_index = 0;
		let mut packet_end_index = 0;

		let mut packet_len = 0;

		self.datastream_provider.read(&mut self.buffer).await;
		// Searches for start of a frame in datastream

		if !init_found {
			let sync_bytes = seek_bytes([SYNC_BIT_1, SYNC_BIT_2].as_ref(), &self.buffer);
			// panic!("SB {}", sync_bytes.unwrap());
			if sync_bytes.is_ok() {
				packet_start_index = sync_bytes.unwrap();
				init_found = true;
			}
		}

		// Searches for end of frame in datastream
		if init_found {
			packet_len = (((self.buffer[packet_start_index + LENGTH_OFFSET_HIGH] as u16) << 8)
				| (self.buffer[packet_start_index + LENGTH_OFFSET_LOW] as u16)) as usize;
			frame_captured = true;
		}

		// Checks if the end bit was missed and is being read past the max possible frame size (4096)

		if frame_captured {
			let frame =
				SbgFrameFactory::new_raw(&self.buffer[packet_start_index..packet_start_index + PRE_DATA_OFFSET_TRUE + packet_len + END_BIT_OFFSET]);
			if self.check_crc(&frame) { Ok(frame) } else { Err(FrameError::CRCBad) }
		} else {
			Err(FrameError::NoFrameFound)
		}
	}

	pub async fn read_frame_by_msgid(
		&mut self,
		id: MESSAGE,
	) -> Result<FrameTypes, FrameError> {
		self.buffer_index = 0;

		// Variables for seeking
		let mut init_found = false;
		let mut frame_captured = false;
		let mut buffer_empty = false;
		let mut bail_count = 0;

		let mut packet_start_index = 0;
		let mut packet_end_index = 0;
		let mut offset = 0;

		let mut last_packet_start_index: isize = -1;
		let mut last_packet_end_index: isize = -1;

		let mut packet_len = 0;

		self.datastream_provider.read(&mut self.buffer).await;

		// Loops until a frame is found, buffer is empty (nothing has desired id), or the bail condition is met (if there is too much noise, just request a fresh stream)
		while !frame_captured && bail_count < BAIL_FROM_LOOP && !buffer_empty {
			init_found = false;
			// Searches for start of a frame in datastream
			if !init_found {
				let sync_bytes = seek_bytes([SYNC_BIT_1, SYNC_BIT_2].as_ref(), &self.buffer[packet_start_index..]);

				if sync_bytes.is_ok() {
					packet_start_index = sync_bytes.unwrap();
					init_found = true;
				} else {
					buffer_empty = true;
				}
			}

			// Checks if frame has correct message id
			if init_found {
				if (self.buffer[packet_start_index + MESSAGE_FIELD_OFFSET] != id.clone() as u8) {
					init_found = false;
					// let packet_len = ((slice_to_check[LENGTH_OFFSET_HIGH] as u16) | (slice_to_check[LENGTH_OFFSET_LOW] as u16) << 8) as usize;
					offset += 2;
				} else {
					packet_len = (((self.buffer[packet_start_index + LENGTH_OFFSET_HIGH] as u16) << 8)
						| (self.buffer[packet_start_index + LENGTH_OFFSET_LOW] as u16)) as usize;

					frame_captured = true;
				}
			}
			bail_count += 1;
			//
		}
		// Checks if the end bit was missed and is being read past the max possible frame size (4096)
		if frame_captured {
			let frame =
				SbgFrameFactory::new_raw(&self.buffer[packet_start_index..packet_start_index + PRE_DATA_OFFSET_TRUE + packet_len + END_BIT_OFFSET]);
			if self.check_crc(&frame) {
				Ok(frame)
			} else {
				// TODO: Errors out. Update to continue seeking as a bad crc doesn't mean the rest of the data is useless;
				Err(FrameError::CRCBad)
			}
		} else {
			Err(FrameError::NoFrameFound)
		}
	}

	pub async fn write_frame(
		&mut self,
		frame: (impl SbgCommand + NoUninit),
	) {
		// The payload for data section in frame
		let payload = bytemuck::bytes_of(&frame);

		let crc = self.generate_crc(
			frame.msg_number(),
			frame_identifiers::CLASS::SBG_ECOM_CLASS_CMD_0 as u8,
			payload.len() as u16,
			payload,
		);

		// I need to look into why the buffer needs to be owned, I honestly forgot :)
		let payload_owned = generate_owned_buffer(payload);

		let frame = SbgFrameStandard::new(
			frame.msg_number(),
			frame_identifiers::CLASS::SBG_ECOM_CLASS_CMD_0,
			payload.len() as u16,
			payload_owned,
			crc,
		);

		let mut data: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

		frame.serialize(&mut data);
		self.datastream_provider.write(&data).await;
	}

	pub fn transmit() {}
}


use std::path::Prefix;

use defmt::warn;
// use defmt::{error, warn};
use embedded_io_async::{Read, Write};
use crate::sbg_frame::{FrameTypes, SbgFrameFactory};

pub const BUFFER_SIZE: usize = 4096;
pub const SYNC_BIT_1: u8 = 0xFF;
pub const SYNC_BIT_2: u8 = 0x5A;
pub const END_BIT: u8 = 0x33;

pub const PRE_DATA_OFFSET: usize = 4;

pub struct SbgDevice<'a, U, T>{   
    datastream_provider: U,
    crc_provider: T,
    pub buffer: &'a mut[u8],
    buffer_index: usize,
    buffer_max: usize
}

#[derive(Debug)]
pub enum SeekError {
    FailedToFindBytes
}

fn seek_bytes(bytes_to_find: &[u8], buffer: &[u8]) -> Result<usize, SeekError> {
    let mut tally_of_found = 0;
    let mut index_of_bit = 0;
    let mut bit_tgt = bytes_to_find[0];
    for i in 0..buffer.len() {
        if buffer[i] == bit_tgt {
            tally_of_found += 1;
            
            if tally_of_found == bytes_to_find.len() {
                index_of_bit = i - bytes_to_find.len() + 1;
                return Ok(index_of_bit);
            }

            bit_tgt = bytes_to_find[tally_of_found];
        }
        else {
            tally_of_found = 0;
        }
    }
    return Err(SeekError::FailedToFindBytes);
}



impl <'a, U, T> SbgDevice<'a, U,T> where U: Read + Write, T: Fn(&[u8]) -> u16 {
        pub fn new(uart: U, crc_provider: T, buffer: &'a mut[u8]) -> SbgDevice<'a, U, T> {
            let max = buffer.len();
            SbgDevice {datastream_provider: uart, crc_provider: crc_provider, buffer: buffer, buffer_index: 0, buffer_max: max}
        }
        
        fn append_to_internal_buffer(&mut self, data_to_append:&[u8]){
            for i in data_to_append {
                // Checks if buffer is overflowing and resets to 0
                if self.buffer_index >= self.buffer_max - 1{
                    warn!("SBG buffer overflow!");
                    self.buffer_index = 0;        
                }
                self.buffer[self.buffer_index] = i.clone();
                self.buffer_index += 1;
            }       
        }

        // Checks the provided frame's CRC for data integrity
        fn check_crc(&self, frame: &FrameTypes) -> bool {
            let mut buffer = [0; 4096];
            let frame_data = frame.get_data();
            let trimmed_data: &[u8];


            // Appends frame's message id, class, and length. See: <https://developer.sbg-systems.com/sbgECom/5.3/md_doc_2binary_protocol.html#crcDefinition>
            buffer[0] = frame.get_msgid();
            buffer[1] = frame.get_class() as u8;
            buffer[2] = (frame.get_length()) as u8;
            buffer[3] = (frame.get_length() >> 8) as u8;


            for i in 0..frame_data.len() {
                buffer[PRE_DATA_OFFSET + i] = frame_data[i];
            }

            // Trim out placeholder 0s for CRC 
            trimmed_data = &buffer[0..(frame.get_length() as usize + PRE_DATA_OFFSET)];

            (self.crc_provider)(trimmed_data) == frame.get_crc()
        }


           
        pub async fn read_frame(&mut self) -> FrameTypes{
            // Initializes a buffer for the read data. Locates sync bits 0xFF and 0x5A

            let mut init_found = false;
            let mut frame_captured = false;
            
            // Searches for the start and end of the package 
            while !frame_captured {
                let mut temp:[u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
                let mut packet_start_index = 0;
                let mut packet_end_index = 0;

                self.datastream_provider.read(&mut temp).await;

                // Searches for start of a frame in datastream
                if !init_found{
                    let sync_bytes  = seek_bytes([SYNC_BIT_1, SYNC_BIT_2].as_ref(), &temp);
                    if sync_bytes.is_ok() {
                        packet_start_index = sync_bytes.unwrap();
                        init_found = true;
                    }
                }

                // Searches for end of frame in datastream
                if init_found {
                    let end_bytes = seek_bytes([END_BIT].as_ref(), &temp);
                    if end_bytes.is_ok() {
                        packet_end_index = end_bytes.unwrap();
                        frame_captured = true;
                    }
                    else {
                    packet_end_index = BUFFER_SIZE
                    }
                    self.append_to_internal_buffer(&temp[packet_start_index..packet_end_index]);
                }
            }
 
            SbgFrameFactory::new_raw(&self.buffer)
        }
        
        pub fn transmit() {}
    }
    


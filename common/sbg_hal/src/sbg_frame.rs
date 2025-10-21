

use crate::data_structs::frame_identifiers::*;

pub const CLASS_IDENTIFIER_MASK: u8 = 0b10000000;
pub const MESSAGE_FIELD_OFFSET: usize = 2;
pub const CLASS_FIELD_OFFSET: usize = 3;
pub const LENGTH_OFFSET_HIGH: usize = 5;
pub const LENGTH_OFFSET_LOW: usize = 4;


pub const FRAME_OFFSET_STANDARD: usize = 6;
pub const CRC_OFFSET_HIGH_STANDARD: usize = 6;
pub const CRC_OFFSET_LOW_STANDARD: usize = 7;


pub const TX_ID_OFFSET_EXTENDED: usize = 6;
pub const PAGE_IDX_OFFSET_EXTENDED_HIGH: usize = 7;
pub const PAGE_IDX_OFFSET_EXTENDED_LOW: usize = 8;
pub const NR_PAGES_OFFSET_EXTENDED_HIGH: usize = 9;
pub const NR_PAGES_OFFSET_EXTENDED_LOW: usize = 10;
pub const FRAME_OFFSET_EXTENDED: usize = 11;
pub const CRC_OFFSET_HIGH_EXTENDED: usize = 12;
pub const CRC_OFFSET_LOW_EXTENDED: usize = 13;


pub enum FrameTypes {
    Standard(SbgFrameStandard),
    Extended(SbgFrameExtended)
}


impl FrameTypes {
    pub fn read_data(&self) -> [u8; 4086] {
        match self {
            FrameTypes::Standard(frame) => return frame.data.data.clone(),
            FrameTypes::Extended(frame) => return frame.data.data.clone(),
        }
    }
    pub fn get_msgid(&self) -> u8 {
            match self {
            FrameTypes::Standard(frame) => return frame.msg.clone(),
            FrameTypes::Extended(frame) => return frame.msg.clone(),
        }
    }
    pub fn get_class(&self) -> CLASS {
            match self {
            FrameTypes::Standard(frame) => return frame.class.clone(),
            FrameTypes::Extended(frame) => return frame.class.clone(),
        }
    }
    pub fn get_length(&self) -> u16 {
            match self {
            FrameTypes::Standard(frame) => return frame.length.clone(),
            FrameTypes::Extended(frame) => return frame.length.clone(),
        }
    }
    pub fn get_data(&self) -> [u8; 4086] {
            match self {
            FrameTypes::Standard(frame) => return frame.data.data.clone(),
            FrameTypes::Extended(frame) => return frame.data.data.clone(),
        }
    }
    pub fn get_crc(&self) -> u16 {
            match self {
            FrameTypes::Standard(frame) => return frame.crc.clone(),
            FrameTypes::Extended(frame) => return frame.crc.clone(),
        }
    }
    
}

impl Default for FrameTypes {
    fn default() -> Self {
        Self::Standard(SbgFrameStandard::default())
    }
}

pub struct SbgFrameFactory;

// Factory to generate frames for later usage
impl SbgFrameFactory {
    pub fn new_raw(data: &[u8] ) -> FrameTypes {
        let class_identifier = &data[3];

        if (class_identifier & CLASS_IDENTIFIER_MASK) == CLASS_IDENTIFIER_MASK {
            let mut frame = SbgFrameExtended::default();

            // Sets all of the extended frame's fields
            frame.msg = data[2].clone();
            frame.class = CLASS::from(data[CLASS_FIELD_OFFSET] & (!CLASS_IDENTIFIER_MASK));
            frame.length = (data[LENGTH_OFFSET_HIGH] as u16) | (data[LENGTH_OFFSET_LOW] as u16) << 8;
            frame.tx_id = data[TX_ID_OFFSET_EXTENDED];
            frame.nr_pages = (data[NR_PAGES_OFFSET_EXTENDED_LOW] as u16) | (data[NR_PAGES_OFFSET_EXTENDED_HIGH] as u16) << 8;
            frame.page_index = (data[PAGE_IDX_OFFSET_EXTENDED_LOW] as u16) | (data[PAGE_IDX_OFFSET_EXTENDED_HIGH] as u16) << 8;
            for i in FRAME_OFFSET_EXTENDED..(frame.length + FRAME_OFFSET_EXTENDED as u16) as usize{
                frame.data.data[i - FRAME_OFFSET_EXTENDED] = data[i];
            }
            frame.crc = data[CRC_OFFSET_LOW_EXTENDED + frame.length as usize] as u16 |  (data[CRC_OFFSET_HIGH_EXTENDED + frame.length as usize] as u16) << 8; 
            
            return FrameTypes::Extended(frame);
        }
        else {
            let mut frame = SbgFrameStandard::default();

            // Sets all of the standard frame's fields
            frame.msg = data[2].clone();
            frame.class = CLASS::from(data[CLASS_FIELD_OFFSET] & (!CLASS_IDENTIFIER_MASK));
            frame.length = (data[LENGTH_OFFSET_LOW] as u16) | (data[LENGTH_OFFSET_HIGH] as u16) << 8;
            for i in FRAME_OFFSET_STANDARD..(frame.length + FRAME_OFFSET_STANDARD as u16) as usize{
                frame.data.data[i - FRAME_OFFSET_STANDARD] = data[i];
            }
            frame.crc = data[CRC_OFFSET_LOW_STANDARD + frame.length as usize] as u16 |  (data[CRC_OFFSET_HIGH_STANDARD + frame.length as usize] as u16) << 8; 

            
            return FrameTypes::Standard(frame);
        }
    }
}

// Data intermediate as I couldn't get the SbgFrameStandard and SbgFrameExtended structs to provide a default to [u8; 4086]. I need to look for fix or better workaround
struct DATA {
    pub data: [u8; 4086]
}

impl Default for DATA {
    fn default() -> Self {
        Self { data: [0; 4086] }
    }
}

impl DATA {
    pub fn new(data: [u8; 4086]) -> DATA{
        DATA { data: data }
    }
}

#[derive(Default)]
struct SbgFrameStandard {
    msg: u8,
    class: CLASS,
    length: u16,
    data: DATA,
    crc: u16
}

impl SbgFrameStandard {
    pub fn new(msg: u8, class: CLASS, length: u16, data: [u8; 4086], crc: u16) -> SbgFrameStandard {
        SbgFrameStandard { msg: msg, class: class, length: length, data: DATA::new(data), crc: crc}
    }
}

#[derive(Default)]
struct SbgFrameExtended {
    msg: u8,
    class: CLASS,
    length: u16,
    tx_id: u8,
    page_index: u16,
    nr_pages: u16,
    data: DATA,
    crc: u16
}

impl SbgFrameExtended {
    pub fn new(msg: u8, class: CLASS, length: u16,  tx_id: u8, page_index: u16, nr_pages: u16, data: [u8; 4086], crc: u16) -> SbgFrameExtended {
        SbgFrameExtended { msg: msg, class: class, length: length, tx_id: tx_id, page_index: page_index, nr_pages: nr_pages, data: DATA::new(data), crc: crc}
    }
}

#![allow(non_camel_case_types)]

// Frame's message ID
#[derive(Clone, Debug)]
pub enum CLASS {
    SBG_ECOM_CLASS_LOG_ECOM_0 = 0x00,
    SBG_ECOM_CLASS_LOG_ECOM_1 = 0x01,
    SBG_ECOM_CLASS_LOG_NMEA_0 = 0x02,
    SBG_ECOM_CLASS_LOG_NMEA_1 = 0x03,
    SBG_ECOM_CLASS_LOG_THIRD_PARTY_0 = 0x04,
    SBG_ECOM_CLASS_LOG_NMEA_GNSS = 0x05,
    SBG_ECOM_CLASS_CMD_0 = 0x10,
}

impl Default for CLASS {
    fn default() -> Self {
        CLASS::SBG_ECOM_CLASS_LOG_ECOM_0
    }
}

impl From<u8> for CLASS {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::SBG_ECOM_CLASS_LOG_ECOM_0,
            0x01 => Self::SBG_ECOM_CLASS_LOG_ECOM_1,
            0x02 => Self::SBG_ECOM_CLASS_LOG_NMEA_0,
            0x03 => Self::SBG_ECOM_CLASS_LOG_NMEA_1,
            0x04 => Self::SBG_ECOM_CLASS_LOG_THIRD_PARTY_0,
            0x05 => Self::SBG_ECOM_CLASS_LOG_NMEA_GNSS,
            0x10 => Self::SBG_ECOM_CLASS_CMD_0,
            _ => Self::SBG_ECOM_CLASS_LOG_ECOM_0
        }
    }
}

use bitflags::bitflags;

#[repr(u8)]
pub enum Register {
    ID = 0x00,
    POWER = 0x01,
    INTERFACE = 0x02,
    MODE0 = 0x03,
    MODE1 = 0x04,
    MODE2 = 0x05,
    INPMUX = 0x06,
    OFCAL0 = 0x07,
    OFCAL1 = 0x08,
    OFCAL2 = 0x09,
    FSCAL0 = 0x0A,
    FSCAL1 = 0x0B,
    FSCAL2 = 0x0C,
    IDACMUX = 0x0D,
    IDACMAG = 0x0E,
    REFMUX = 0x0F,
    TDACP = 0x10,
    TDACN = 0x11,
    GPIOCON = 0x12,
    GPIODIR = 0x13,
    GPIODAT = 0x14,
    ADC2CFG = 0x15,
    ADC2MUX = 0x16,
    ADC2OFC0 = 0x17,
    ADC2OFC1 = 0x18,
    ADC2FSC0 = 0x19,
    ADC2FSC1 = 0x1A,
}

bitflags! {
    pub struct IdRegister: u8 {
        const _ = !0; // Source may set any bits
    }
}

pub enum DevId {
    ADS1262 = 0b000,
    ADS1263 = 0b001,
}

impl IdRegister {
    pub fn get_rev_id(&self) -> u8 {
        self.bits() & 0b0001_1111
    }

    pub fn get_dev_id(&self) -> DevId {
        match (self.bits() & 0b1110_0000) >> 5 {
            0b000 => DevId::ADS1262,
            0b001 => DevId::ADS1263,

            _ => panic!("Device ID must be 0b000 or 0b001."),
        }
    }
}

bitflags! {
    pub struct PowerRegister: u8 {
        const INTREF = 0b0000_0001;
        const VBIAS  = 0b0000_0010;
        const RESET  = 0b0001_0000;
    }
}

bitflags! {
    pub struct InterfaceRegister: u8 {
        const STATUS  = 0b0000_0100;
        const TIMEOUT = 0b0000_1000;

        const _ = 0b0000_0011; // Source may set CDC bits
    }
}

pub enum Crc {
    DISABLED = 0b00,
    ENABLED = 0b01,
}

impl InterfaceRegister {
    pub fn get_crc(&self) -> Crc {
        match self.bits() & 0b0000_0011 {
            0b00 => Crc::DISABLED,
            0b01 => Crc::ENABLED,

            0b11 => panic!("Reserved state is set. Should not be 0b11."),

            // Exhaustive list for possible combinations of 2 bits
            _ => unreachable!("Only 2 bits should be set.")
        }
    }

    pub fn set_crc(&mut self, crc: Crc) {
        let crc_bits = crc as u8 & 0b0000_0011;
        self.insert(InterfaceRegister::from_bits_truncate(crc_bits));
    }
}

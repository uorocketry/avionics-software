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
    pub struct StatusRegister: u8 {
        const ADC2      = 0b1000_0000;
        const ADC1      = 0b0100_0000;
        const EXTCLK    = 0b0010_0000;
        const REF_ALM   = 0b0001_0000;
        const PGAL_ALM  = 0b0000_1000;
        const PGAH_ALM  = 0b0000_0100;
        const PGAD_ALM  = 0b0000_0010;
        const RESET     = 0b0000_0001;
    }
}

impl StatusRegister {
    pub fn is_adc2_data_new(&self) -> bool {
        self.contains(StatusRegister::ADC2)
    }

    pub fn is_adc1_data_new(&self) -> bool {
        self.contains(StatusRegister::ADC1)
    }

    pub fn is_extclk(&self) -> bool {
        self.contains(StatusRegister::EXTCLK)
    }

    pub fn is_ref_alm(&self) -> bool {
        self.contains(StatusRegister::REF_ALM)
    }

    pub fn is_pgal_alm(&self) -> bool {
        self.contains(StatusRegister::PGAL_ALM)
    }

    pub fn is_pgah_alm(&self) -> bool {
        self.contains(StatusRegister::PGAH_ALM)
    }

    pub fn is_pgad_alm(&self) -> bool {
        self.contains(StatusRegister::PGAD_ALM)
    }

    pub fn is_reset(&self) -> bool {
        self.contains(StatusRegister::RESET)
    }
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
    /// WARNING: If CRC is 0b11 set by ADC, it will reflect as CRC enabled not reserved.
    /// CRC only accounts for 0b00 disabled and 0b01 enabled.
    pub struct InterfaceRegister: u8 {
        const CRC     = 0b0000_0001;
        const STATUS  = 0b0000_0100;
        const TIMEOUT = 0b0000_1000;
    }
}

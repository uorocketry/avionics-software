mod enums;

pub use enums::*;
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

bitflags! {
    pub struct Mode0Register: u8 {
        const RUNMODE = 0b0100_0000;
        const REFREV  = 0b1000_0000;

        const _ = !0; // Source may set any bits
    }
}

impl Mode0Register {
    pub fn get_delay(&self) -> ConversionDelay {
        match self.bits() & 0b0000_1111 {
            0b0000 => ConversionDelay::DNone,
            0b0001 => ConversionDelay::D8_7us,
            0b0010 => ConversionDelay::D17us,
            0b0011 => ConversionDelay::D35us,
            0b0100 => ConversionDelay::D69us,
            0b0101 => ConversionDelay::D139us,
            0b0110 => ConversionDelay::D278us,
            0b0111 => ConversionDelay::D555us,
            0b1000 => ConversionDelay::D1_1ms,
            0b1001 => ConversionDelay::D2_2ms,
            0b1010 => ConversionDelay::D4_4ms,
            0b1011 => ConversionDelay::D8_8ms,

            0b1100..=0b1111 => panic!("Unknown conversion delay"),
            _ => unreachable!(),
        }
    }

    pub fn set_delay(&mut self, delay: ConversionDelay) {
        let bits = delay as u8;
        self.insert(Mode0Register::from_bits_retain(bits));
    }

    pub fn get_chop(&self) -> ChopMode {
        match (self.bits() & 0b0011_0000) >> 4 {
            0b00 => ChopMode::Disabled,
            0b01 => ChopMode::InChopEnabled,
            0b10 => ChopMode::IdacEnabled,
            0b11 => ChopMode::InChopAndIdacEnabled,

            _ => unreachable!(),
        }
    }

    pub fn set_chop(&mut self, chop: ChopMode) {
        let bits = chop as u8;
        self.insert(Mode0Register::from_bits_retain(bits << 4));
    }
}

bitflags! {
    pub struct Mode1Register: u8 {
        const SBPOL = 0b0000_1000;
        const SBADC = 0b0001_0000;

        const _ = !0; // Source may set any bits
    }
}

impl Mode1Register {
    pub fn get_sbmag(&self) -> SensorBiasMagnitude {
        match self.bits() & 0b0000_0111 {
            0b000 => SensorBiasMagnitude::BNone,
            0b001 => SensorBiasMagnitude::B0_5uA,
            0b010 => SensorBiasMagnitude::B2uA,
            0b011 => SensorBiasMagnitude::B10uA,
            0b100 => SensorBiasMagnitude::B50uA,
            0b101 => SensorBiasMagnitude::B200uA,
            0b110 => SensorBiasMagnitude::R10MOhm,

            0b111 => panic!("Reserved SBMAG"),
            _ => unreachable!()
        }
    }

    pub fn set_sbmag(&mut self, sbmag: SensorBiasMagnitude) {
        let bits = sbmag as u8;
        self.insert(Mode1Register::from_bits_retain(bits));
    }

    pub fn get_filter(&self) -> DigitalFilter {
        match (self.bits() & 0b1110_0000) >> 5 {
            0b000 => DigitalFilter::Sinc1,
            0b001 => DigitalFilter::Sinc2,
            0b010 => DigitalFilter::Sinc3,
            0b011 => DigitalFilter::Sinc4,
            0b100 => DigitalFilter::FIR,

            0b101..=0b111 => panic!("Reserved filter"),
            _ => unreachable!()
        }
    }

    pub fn set_filter(&mut self, filter: DigitalFilter) {
        let bits = filter as u8;
        self.insert(Mode1Register::from_bits_retain(bits << 5));
    }
}

bitflags! {
    pub struct Mode2Register: u8 {
        const BYPASS = 0b1000_0000;

        const _ = !0; // Source may set any bits
    }
}

impl Mode2Register {
    pub fn get_dr(&self) -> DataRate {
        match self.bits() & 0b0000_1111 {
            0b0000 => DataRate::SPS2_5,
            0b0001 => DataRate::SPS5,
            0b0010 => DataRate::SPS10,
            0b0011 => DataRate::SPS16_6,
            0b0100 => DataRate::SPS20,
            0b0101 => DataRate::SPS50,
            0b0110 => DataRate::SPS60,
            0b0111 => DataRate::SPS100,
            0b1000 => DataRate::SPS400,
            0b1001 => DataRate::SPS1200,
            0b1010 => DataRate::SPS2400,
            0b1011 => DataRate::SPS4800,
            0b1100 => DataRate::SPS7200,
            0b1101 => DataRate::SPS14400,
            0b1110 => DataRate::SPS19200,
            0b1111 => DataRate::SPS38400,

            _ => unreachable!()
        }
    }

    pub fn set_dr(&mut self, rate: DataRate) {
        let bits = rate as u8;
        self.insert(Mode2Register::from_bits_retain(bits));
    }

    pub fn get_gain(&self) -> PGAGain {
        match (self.bits() & 0b0111_0000) >> 4 {
            0b000 => PGAGain::VV1,
            0b001 => PGAGain::VV2,
            0b010 => PGAGain::VV4,
            0b011 => PGAGain::VV8,
            0b100 => PGAGain::VV16,
            0b101 => PGAGain::VV32,

            0b110 | 0b111 => panic!("Reserved gain"),
            _ => unreachable!()
        }
    }

    pub fn set_gain(&mut self, gain: PGAGain) {
        let bits = gain as u8;
        self.insert(Mode2Register::from_bits_retain(bits << 4));
    }
}

pub enum DevId {
    ADS1262 = 0b000,
    ADS1263 = 0b001,
}

/// Conversion delays follow the pattern `D<len><units>`.
/// - `len` is the length of time where _ is a substitute for a decimal point.
/// - `units` are the units of time where us is microseconds and ms is milliseconds.
/// 
/// D8_7us = delay of 8.7 microseconds. D8_8ms = delay of 8.8 milliseconds.
#[repr(u8)]
pub enum ConversionDelay {
    DNone  = 0b0000,
    D8_7us = 0b0001,
    D17us  = 0b0010,
    D35us  = 0b0011,
    D69us  = 0b0100,
    D139us = 0b0101,
    D278us = 0b0110,
    D555us = 0b0111,
    D1_1ms = 0b1000,
    D2_2ms = 0b1001,
    D4_4ms = 0b1010,
    D8_8ms = 0b1011,
}

impl ConversionDelay {
    /// Returns the delay in nanoseconds.
    pub fn delay(&self) -> u32 {
        match self {
            Self::DNone  => 0,
            Self::D8_7us => 8_700,
            Self::D17us  => 17_000,
            Self::D35us  => 35_000,
            Self::D69us  => 69_000,
            Self::D139us => 139_000,
            Self::D278us => 278_000,
            Self::D555us => 555_000,
            Self::D1_1ms => 1_100_000,
            Self::D2_2ms => 2_200_000,
            Self::D4_4ms => 4_400_000,
            Self::D8_8ms => 8_800_000,
        }
    }
}

#[repr(u8)]
pub enum ChopMode {
    Disabled             = 0b00,
    InChopEnabled        = 0b01,
    IdacEnabled          = 0b10,
    InChopAndIdacEnabled = 0b11,
}

/// SBMAGs follow the pattern `B<mag><units>` or `R<mag><units>`.
/// - `mag` is the magnitude of current or resistance where _ is a substitute for a decimal point.
/// - `units` are the units of current or resistance where uA is microamperes and MOhm is megaohms.
/// 
/// B0_5uA = 0.5 microamps of current. R10MOhm = resistance of 10 megaohms.
#[repr(u8)]
pub enum SensorBiasMagnitude {
    BNone   = 0b000,
    B0_5uA  = 0b001,
    B2uA    = 0b010,
    B10uA   = 0b011,
    B50uA   = 0b100,
    B200uA  = 0b101,
    R10MOhm = 0b110,
}

#[repr(u8)]
pub enum DigitalFilter {
    Sinc1 = 0b000,
    Sinc2 = 0b001,
    Sinc3 = 0b010,
    Sinc4 = 0b011,
    FIR   = 0b100,
}

/// Data rates follow the pattern `SPS<num>`.
/// - `num` is the SPS rate where _ is a substitute for a decimal point.
/// 
/// SPS2_5 = 2.5 SPS.
#[repr(u8)]
pub enum DataRate {
    SPS2_5   = 0b0000,
    SPS5     = 0b0001,
    SPS10    = 0b0010,
    SPS16_6  = 0b0011, // 16.6666... = 50/3
    SPS20    = 0b0100,
    SPS50    = 0b0101,
    SPS60    = 0b0110,
    SPS100   = 0b0111,
    SPS400   = 0b1000,
    SPS1200  = 0b1001,
    SPS2400  = 0b1010,
    SPS4800  = 0b1011,
    SPS7200  = 0b1100,
    SPS14400 = 0b1101,
    SPS19200 = 0b1110,
    SPS38400 = 0b1111,
}

#[repr(u8)]
pub enum PGAGain {
    VV1  = 0b000,
    VV2  = 0b001,
    VV4  = 0b010,
    VV8  = 0b011,
    VV16 = 0b100,
    VV32 = 0b101,
}

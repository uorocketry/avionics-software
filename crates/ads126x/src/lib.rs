#![no_std]
#![no_main]

mod error;

use error::ADS126xError;

use bitflags::bitflags;
use embedded_hal::spi::FullDuplex;
use heapless::Vec;

pub struct ADS126x<SPI>
where
    SPI: FullDuplex<u8>,
{
    spi: SPI,
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

pub enum ADCCommand {
    NOP,
    RESET,
    START1,
    STOP1,
    START2,
    STOP2,
    RDATA1,
    RDATA2,
    SYOCAL1,
    SYGCAL1,
    SFOCAL1,
    SYOCAL2,
    SYGCAL2,
    SFOCAL2,
    RREG(Register, u8), // (register address, number of registers)
    WREG(Register, u8), // (register address, number of registers)
}

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

impl<SPI> ADS126x<SPI>
where
    SPI: FullDuplex<u8>,
{
    pub fn new(spi: SPI) -> Self {
        Self { spi }
    }

    pub fn send_command(&mut self, command: ADCCommand) -> Result<(), ADS126xError> {
        let (opcode1, opcode2) = match command {
            ADCCommand::NOP => (0x00, None),
            ADCCommand::RESET => (0x06, None),
            ADCCommand::START1 => (0x08, None),
            ADCCommand::STOP1 => (0x0A, None),
            ADCCommand::START2 => (0x0C, None),
            ADCCommand::STOP2 => (0x0E, None),
            ADCCommand::RDATA1 => (0x12, None),
            ADCCommand::RDATA2 => (0x14, None),
            ADCCommand::SYOCAL1 => (0x16, None),
            ADCCommand::SYGCAL1 => (0x17, None),
            ADCCommand::SFOCAL1 => (0x19, None),
            ADCCommand::SYOCAL2 => (0x1B, None),
            ADCCommand::SYGCAL2 => (0x1C, None),
            ADCCommand::SFOCAL2 => (0x1E, None),
            ADCCommand::RREG(addr, num) => (0x20 | addr as u8, Some(num)),
            ADCCommand::WREG(addr, num) => (0x40 | addr as u8, Some(num)),
        };

        self.spi.send(opcode1).map_err(|_| ADS126xError::IO)?;
        if let Some(op2) = opcode2 {
            self.spi.send(op2).map_err(|_| ADS126xError::IO)?;
        }
        Ok(())
    }

    pub fn read_register(&mut self, reg: Register, num: u8) -> Result<Vec<u8, 27>, ADS126xError> {
        if num > 27 {
            return Err(ADS126xError::InvalidInputData);
        }
        self.send_command(ADCCommand::RREG(reg, num - 1))?;
        let mut buffer: Vec<u8, 27> = Vec::new();
        for _ in 0..num {
            buffer
                .push(self.spi.read().map_err(|_| ADS126xError::IO)?)
                .map_err(|_| ADS126xError::InvalidInputData)?;
        }
        Ok(buffer)
    }

    pub fn write_register(&mut self, reg: Register, data: &[u8]) -> Result<(), ADS126xError> {
        if data.len() > 27 {
            return Err(ADS126xError::InvalidInputData);
        }
        self.send_command(ADCCommand::WREG(reg, data.len() as u8 - 1))?;
        for &byte in data {
            self.spi.send(byte).map_err(|_| ADS126xError::IO)?;
        }
        Ok(())
    }
}

#![no_std]
#![no_main]

mod error;
mod register;

use error::ADS126xError;
use register::{Register, IdRegister, PowerRegister, InterfaceRegister};

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

    pub fn read_multiple_registers(&mut self, reg: Register, num: u8) -> Result<Vec<u8, 27>, ADS126xError> {
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

    /// Reads data from only the single provided register.
    /// To read multiple registers, see [read_register](ADS126x::read_register).
    pub fn read_register(&mut self, reg: Register) -> Result<u8, ADS126xError> {
        // zero since number of registers read - 1, so 1-1=0. 
        self.send_command(ADCCommand::RREG(reg, 0))?; 
        let data = self.spi.read().map_err(|_| ADS126xError::IO)?;
        Ok(data)
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

    pub fn get_id(&mut self) -> Result<IdRegister, ADS126xError> {
        let bits = self.read_register(Register::ID)?;
        let data = IdRegister::from_bits(bits);
        match data {
            Some(reg) => Ok(reg),
            None => Err(ADS126xError::InvalidInputData),
        }
    }

    pub fn get_power(&mut self) -> Result<PowerRegister, ADS126xError> {
        let bits = self.read_register(Register::POWER)?;
        let data = PowerRegister::from_bits(bits);
        match data {
            Some(reg) => Ok(reg),
            None => Err(ADS126xError::InvalidInputData),
        }
    }

    pub fn set_power(&mut self, reg: &PowerRegister) -> Result<(), ADS126xError> {
        self.write_register(Register::POWER, &[reg.bits()])
    }

    pub fn get_interface(&mut self) -> Result<InterfaceRegister, ADS126xError> {
        let bits = self.read_register(Register::INTERFACE)?;
        let data = InterfaceRegister::from_bits(bits);
        match data {
            Some(reg) => Ok(reg),
            None => Err(ADS126xError::InvalidInputData),
        }
    }

    pub fn set_interface(&mut self, reg: &InterfaceRegister) -> Result<(), ADS126xError> {
        self.write_register(Register::INTERFACE, &[reg.bits()])
    }
}

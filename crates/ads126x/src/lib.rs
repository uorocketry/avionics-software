#![no_std]
#![no_main]

mod error;
mod register;

use error::ADS126xError;
use register::{
    IdRegister,
    InterfaceRegister,
    Mode0Register,
    Mode1Register,
    Mode2Register,
    PowerRegister,
    Register
};

use embedded_hal::spi::FullDuplex;
use heapless::Vec;

/// The [`Result`] type for ADS126x operations.
pub type Result<T> = core::result::Result<T, ADS126xError>;

pub struct ADS126x<SPI>
where
    SPI: FullDuplex<u8>,
{
    spi: SPI,
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

    pub fn send_command(&mut self, command: ADCCommand) -> Result<()> {
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

    /// Reads data from multiple registers starting at the provided register.
    /// To read a single register, see [`ADS126x::read_register`].
    pub fn read_multiple_registers(&mut self, reg: Register, num: u8) -> Result<Vec<u8, 27>> {
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
    /// To read multiple registers, see [`ADS126x::read_multiple_registers`].
    pub fn read_register(&mut self, reg: Register) -> Result<u8> {
        // zero since number of registers read - 1, so 1-1=0. 
        self.send_command(ADCCommand::RREG(reg, 0))?; 
        let data = self.spi.read().map_err(|_| ADS126xError::IO)?;
        Ok(data)
    }

    /// Writes data to multiple registers starting at the provided register.
    /// To write data to a single register, see [`ADS126x::write_register`].
    pub fn write_multiple_registers(&mut self, reg: Register, data: &[u8]) -> Result<()> {
        if data.len() > 27 {
            return Err(ADS126xError::InvalidInputData);
        }
        self.send_command(ADCCommand::WREG(reg, data.len() as u8 - 1))?;
        for &byte in data {
            self.spi.send(byte).map_err(|_| ADS126xError::IO)?;
        }
        Ok(())
    }

    /// Writes data to only the single provided register.
    /// To write data to multiple registers, see [`ADS126x::write_multiple_registers`].
    pub fn write_register(&mut self, reg: Register, data: u8) -> Result<()> {
        self.send_command(ADCCommand::WREG(reg, 0))?;
        self.spi.send(data).map_err(|_| ADS126xError::IO)
    }

    pub fn get_id(&mut self) -> Result<IdRegister> {
        let bits = self.read_register(Register::ID)?;
        let data = IdRegister::from_bits(bits);
        match data {
            Some(reg) => Ok(reg),
            None => Err(ADS126xError::InvalidInputData),
        }
    }

    pub fn get_power(&mut self) -> Result<PowerRegister> {
        let bits = self.read_register(Register::POWER)?;
        let data = PowerRegister::from_bits(bits);
        match data {
            Some(reg) => Ok(reg),
            None => Err(ADS126xError::InvalidInputData),
        }
    }

    pub fn set_power(&mut self, reg: &PowerRegister) -> Result<()> {
        self.write_register(Register::POWER, reg.bits())
    }

    pub fn get_interface(&mut self) -> Result<InterfaceRegister> {
        let bits = self.read_register(Register::INTERFACE)?;
        let data = InterfaceRegister::from_bits(bits);
        match data {
            Some(reg) => Ok(reg),
            None => Err(ADS126xError::InvalidInputData),
        }
    }

    pub fn set_interface(&mut self, reg: &InterfaceRegister) -> Result<()> {
        self.write_register(Register::INTERFACE, reg.bits())
    }

    pub fn get_mode0(&mut self) -> Result<Mode0Register> {
        let bits = self.read_register(Register::MODE0)?;
        let data = Mode0Register::from_bits(bits);
        match data {
            Some(reg) => Ok(reg),
            None => Err(ADS126xError::InvalidInputData),
        }
    }

    pub fn set_mode0(&mut self, reg: &Mode0Register) -> Result<()> {
        self.write_register(Register::MODE0, reg.bits())
    }
    
    pub fn get_mode1(&mut self) -> Result<Mode1Register> {
        let bits = self.read_register(Register::MODE1)?;
        let data = Mode1Register::from_bits(bits);
        match data {
            Some(reg) => Ok(reg),
            None => Err(ADS126xError::InvalidInputData),
        }
    }

    pub fn set_mode1(&mut self, reg: &Mode1Register) -> Result<()> {
        self.write_register(Register::MODE1, reg.bits())
    }
    
    pub fn get_mode2(&mut self) -> Result<Mode2Register> {
        let bits = self.read_register(Register::MODE2)?;
        let data = Mode2Register::from_bits(bits);
        match data {
            Some(reg) => Ok(reg),
            None => Err(ADS126xError::InvalidInputData),
        }
    }

    pub fn set_mode2(&mut self, reg: &Mode2Register) -> Result<()> {
        self.write_register(Register::MODE2, reg.bits())
    }
}

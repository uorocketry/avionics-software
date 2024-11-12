#![no_std]
#![no_main]

mod error;
mod register;

use error::ADS126xError;
use register::{
    IdRegister, IdacMagRegister, IdacMuxRegister, InpMuxRegister, InterfaceRegister, Mode0Register, Mode1Register, Mode2Register, PowerRegister, RefMuxRegister, Register
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
    /// 
    /// Vector returns byte for each register read in order registers were read (increasing address).
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
    /// 
    /// Data has byte for each register in order registers are written to (increasing address).
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

    pub fn get_inpmux(&mut self) -> Result<InpMuxRegister> {
        let bits = self.read_register(Register::INPMUX)?;
        let data = InpMuxRegister::from_bits(bits);
        match data {
            Some(reg) => Ok(reg),
            None => Err(ADS126xError::InvalidInputData),
        }
    }

    pub fn set_inpmux(&mut self, reg: &InpMuxRegister) -> Result<()> {
        self.write_register(Register::INPMUX, reg.bits())
    }

    pub fn get_ofcal(&mut self) -> Result<u32> {
        let mut bytes = self.read_multiple_registers(Register::OFCAL0, 3)?; // [OFCAL0, OFCAL1, OFCAL2]
        bytes.reverse(); // [OFCAL2, OFCAL1, OFCAL0]
        let mut res: u32 = 0;
        for &b in &bytes[3..0] {
            res <<= 8; // Shift previous bits left one byte
            res |= b as u32; // Append new byte to the end
        }
        Ok(res)
    }
    
    pub fn set_ofcal(&mut self, ofcal: u32) -> Result<()> {
        let mut bytes: [u8; 4] = [0; 4];
        for i in 0..4 {
            let b = (ofcal >> (8 * i)) & 0xFF; // Get desired byte
            bytes[i] = u8::try_from(b).unwrap(); // Should not panic as & 0xFF ensures b is a u8
        }
        self.write_multiple_registers(Register::OFCAL0, &bytes[0..3])
    }

    pub fn get_fscal(&mut self) -> Result<u32> {
        let mut bytes = self.read_multiple_registers(Register::FSCAL0, 3)?; // [FSCAL0, FSCAL1, FSCAL2]
        bytes.reverse(); // [FSCAL2, FSCAL1, FSCAL0]
        let mut res: u32 = 0;
        for &b in &bytes[3..0] {
            res <<= 8; // Shift previous bits left one byte
            res |= b as u32; // Append new byte to the end
        }
        Ok(res)
    }
    
    pub fn set_fscal(&mut self, fscal: u32) -> Result<()> {
        let mut bytes: [u8; 4] = [0; 4];
        for i in 0..4 {
            let b = (fscal >> (8 * i)) & 0xFF; // Get desired byte
            bytes[i] = u8::try_from(b).unwrap(); // Should not panic as & 0xFF ensures b is a u8
        }
        self.write_multiple_registers(Register::FSCAL0, &bytes[0..3])
    }

    pub fn get_idacmux(&mut self) -> Result<IdacMuxRegister> {
        let bits = self.read_register(Register::IDACMUX)?;
        let data = IdacMuxRegister::from_bits(bits);
        match data {
            Some(reg) => Ok(reg),
            None => Err(ADS126xError::InvalidInputData),
        }
    }

    pub fn set_idacmux(&mut self, reg: &IdacMuxRegister) -> Result<()> {
        self.write_register(Register::IDACMUX, reg.bits())
    }

    pub fn get_idacmag(&mut self) -> Result<IdacMagRegister> {
        let bits = self.read_register(Register::IDACMAG)?;
        let data = IdacMagRegister::from_bits(bits);
        match data {
            Some(reg) => Ok(reg),
            None => Err(ADS126xError::InvalidInputData),
        }
    }

    pub fn set_idacmag(&mut self, reg: &IdacMagRegister) -> Result<()> {
        self.write_register(Register::IDACMAG, reg.bits())
    }

    pub fn get_refmux(&mut self) -> Result<RefMuxRegister> {
        let bits = self.read_register(Register::REFMUX)?;
        let data = RefMuxRegister::from_bits(bits);
        match data {
            Some(reg) => Ok(reg),
            None => Err(ADS126xError::InvalidInputData),
        }
    }

    pub fn set_refmux(&mut self, reg: &RefMuxRegister) -> Result<()> {
        self.write_register(Register::REFMUX, reg.bits())
    }
}

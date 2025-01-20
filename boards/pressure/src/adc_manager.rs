use ads126x::{
    register::{DataRate, Mode1Register, Mode2Register},
    ADCCommand, Ads126x,
};

use common_arm::spawn;
use stm32h7xx_hal::{
    gpio::{Output, Pin, PushPull},
    spi::Spi,
};
use stm32h7xx_hal::prelude::*;
use defmt::info;
use crate::app::delay;
use stm32h7xx_hal::nb;


// There is an option to use interrupts using the data ready pins, but for now we will poll.
pub struct AdcManager {
    pub spi: Spi<stm32h7xx_hal::pac::SPI4, stm32h7xx_hal::spi::Enabled, u8>,
    pub adc1: Ads126x<Pin<'C', 11, Output<PushPull>>>,
    pub adc2: Ads126x<Pin<'D', 1, Output<PushPull>>>,
    pub adc1_cs: Pin<'C', 10, Output<PushPull>>,
    pub adc2_cs: Pin<'D', 2, Output<PushPull>>,
}

impl AdcManager {
    pub fn new(
        spi: Spi<stm32h7xx_hal::pac::SPI4, stm32h7xx_hal::spi::Enabled, u8>,
        adc1_rst: Pin<'C', 11, Output<PushPull>>,
        adc2_rst: Pin<'D', 1, Output<PushPull>>,
        adc1_cs: Pin<'C', 10, Output<PushPull>>,
        adc2_cs: Pin<'D', 2, Output<PushPull>>,
    ) -> Self {
        Self {
            spi,
            adc1: Ads126x::new(adc1_rst),
            adc2: Ads126x::new(adc2_rst),
            adc1_cs,
            adc2_cs,
        }
    }

    pub fn init_adc1(&mut self, negative: ads126x::register::NegativeInpMux, positive: ads126x::register::PositiveInpMux) -> Result<(), ads126x::error::ADS126xError> {
        self.adc2_cs.set_high();
        self.adc1_cs.set_low();
        self.adc1.reset()?;
        self.adc1_cs.set_high();
        self.adc1_cs.set_low();

        // 2^16 cycles of delay
        // cortex_m::asm::delay(65536);
        cortex_m::asm::delay(250_000_000);


        let mut mode1_cfg = Mode1Register::default();
        mode1_cfg.set_filter(ads126x::register::DigitalFilter::Sinc1);
        self.adc1.set_mode1(&mode1_cfg, &mut self.spi)?;

        let mut mode2_cfg = Mode2Register::default();
        mode2_cfg.set_dr(DataRate::SPS1200);
        self.adc1.set_mode2(&mode2_cfg, &mut self.spi)?;

        let mode1_cfg_real = self.adc1.get_mode1(&mut self.spi)?;
        let mode2_cfg_real = self.adc1.get_mode2(&mut self.spi)?;

        // verify 
        info!("Mode1: {:#010b}", mode1_cfg_real.bits());
        info!("Mode2: {:#010b}", mode2_cfg_real.bits());
        assert!(mode1_cfg.difference(mode1_cfg_real).is_empty()); 
        assert!(mode2_cfg.difference(mode2_cfg_real).is_empty()); 

        let mut reg = ads126x::register::InpMuxRegister::default();
        reg.set_muxn(negative);
        reg.set_muxp(positive);
        self.adc1.set_inpmux(&reg, &mut self.spi)?;

        self.adc1.send_command(ADCCommand::START1, &mut self.spi)?;
        self.adc1.send_command(ADCCommand::START2, &mut self.spi)?;
        cortex_m::asm::delay(250_000_000);

        Ok(())
    }

    pub fn init_adc2(&mut self) -> Result<(), ads126x::error::ADS126xError> {
        self.adc1_cs.set_high();
        self.adc2_cs.set_low();
        self.adc2.reset()?;
        info!("ADC2 reset");

        cortex_m::asm::delay(65536);

        let mut mode1_cfg = Mode1Register::default();
        mode1_cfg.set_filter(ads126x::register::DigitalFilter::Sinc1);
        self.adc1.set_mode1(&mode1_cfg, &mut self.spi)?;
        info!("ADC2 mode1 set");

        let mut mode2_cfg = Mode2Register::default();
        mode2_cfg.set_dr(DataRate::SPS1200);
        self.adc1.set_mode2(&mode2_cfg, &mut self.spi)?;
        info!("ADC2 mode2 set");

        self.adc1.send_command(ADCCommand::START1, &mut self.spi)?;
        info!("ADC2 start1");
        self.adc1.send_command(ADCCommand::START2, &mut self.spi)?;
        info!("ADC2 start2");
        Ok(())
    }

    pub fn read_adc1_data(&mut self, negative: ads126x::register::NegativeInpMux, positive: ads126x::register::PositiveInpMux) -> Result<[u8; 4], ads126x::error::ADS126xError> {
        // check if our inputmux is set to use the right pins. 
        self.adc2_cs.set_high(); // disable adc 2 
        self.adc1_cs.set_low(); // enable adc 1
        // self.adc1.send_command(ads126x::ADCCommand::STOP1, &mut self.spi)?;
        // configure the input mux 
        let mut reg = ads126x::register::InpMuxRegister::default();
        reg.set_muxn(negative);
        reg.set_muxp(positive);
        self.adc1.set_inpmux(&reg, &mut self.spi)?;

        info!("Input mux set");
        // ask for data 
        let status = self.adc1.send_command(ADCCommand::RDATA1, &mut self.spi)?;

        // if status.get_adc1_new() {
            info!("Data requested");
            // read 4 bytes of data from the spi
            let mut data = [0; 4];
            for i in 0..4 {
                data[i] = self.spi.read().map_err(|_| ads126x::error::ADS126xError::IO)?;
                info!("Data read");
            }        

            return Ok(data);
        // }
        
        // info!("Data requested");
        // // read 4 bytes of data from the spi
        // let mut data = [0; 4];
        // for i in 0..2 {
        //     data[i] = self.spi.read().map_err(|_| ads126x::error::ADS126xError::IO)?;
        //     info!("Data read");
        // }        

        Err(ads126x::error::ADS126xError::IO)
    }
}
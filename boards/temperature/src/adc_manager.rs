use ads126x::{
    register::{DataRate, Mode1Register, Mode2Register},
    ADCCommand, Ads126x,
};

use common_arm::spawn;
use stm32h7xx_hal::{
    gpio::{Output, Pin, PushPull},
    spi::Spi,
};

use crate::app::delay;

pub struct AdcManager {
    pub spi: Spi<stm32h7xx_hal::pac::SPI1, stm32h7xx_hal::spi::Enabled, u8>,
    pub adc1: Ads126x<Pin<'A', 0, Output<PushPull>>>,
    pub adc2: Ads126x<Pin<'A', 1, Output<PushPull>>>,
    pub adc1_cs: Pin<'A', 4, Output<PushPull>>,
    pub adc2_cs: Pin<'A', 5, Output<PushPull>>,
}

impl AdcManager {
    pub fn new(
        spi: Spi<stm32h7xx_hal::pac::SPI1, stm32h7xx_hal::spi::Enabled, u8>,
        adc1_pin: Pin<'A', 0, Output<PushPull>>,
        adc2_pin: Pin<'A', 1, Output<PushPull>>,
        adc1_cs: Pin<'A', 4, Output<PushPull>>,
        adc2_cs: Pin<'A', 5, Output<PushPull>>,
    ) -> Self {
        Self {
            spi,
            adc1: Ads126x::new(adc1_pin),
            adc2: Ads126x::new(adc2_pin),
            adc1_cs,
            adc2_cs,
        }
    }

    pub fn init_adc1(&mut self) -> Result<(), ads126x::error::ADS126xError> {
        self.adc2_cs.set_high();
        self.adc1_cs.set_low();
        self.adc1.reset()?;

        spawn!(delay, 1000);

        let mut mode1_cfg = Mode1Register::default();
        mode1_cfg.set_filter(ads126x::register::DigitalFilter::Sinc1);
        self.adc1.set_mode1(&mode1_cfg, &mut self.spi)?;

        let mut mode2_cfg = Mode2Register::default();
        mode2_cfg.set_dr(DataRate::SPS1200);
        self.adc1.set_mode2(&mode2_cfg, &mut self.spi)?;

        self.adc1.send_command(ADCCommand::START1, &mut self.spi)?;
        self.adc1.send_command(ADCCommand::START2, &mut self.spi)?;

        Ok(())
    }

    pub fn init_adc2(&mut self) -> Result<(), ads126x::error::ADS126xError> {
        self.adc1_cs.set_high();
        self.adc2_cs.set_low();
        self.adc2.reset()?;

        spawn!(delay, 1000);

        let mut mode1_cfg = Mode1Register::default();
        mode1_cfg.set_filter(ads126x::register::DigitalFilter::Sinc1);
        self.adc1.set_mode1(&mode1_cfg, &mut self.spi)?;

        let mut mode2_cfg = Mode2Register::default();
        mode2_cfg.set_dr(DataRate::SPS1200);
        self.adc1.set_mode2(&mode2_cfg, &mut self.spi)?;

        self.adc1.send_command(ADCCommand::START1, &mut self.spi)?;
        self.adc1.send_command(ADCCommand::START2, &mut self.spi)?;

        Ok(())
    }
}

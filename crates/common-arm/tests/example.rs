#![no_std]
#![no_main]

use common_arm::SdManager;
use defmt::info;
use panic_probe as _;
use stm32h7xx_hal::gpio::{Output, PushPull, PA4};
use stm32h7xx_hal::pac;
use stm32h7xx_hal::prelude::*;
use stm32h7xx_hal::spi;

#[defmt_test::tests]
mod tests {
    use super::*;

    #[init]
    fn init() {
        let _cp = cortex_m::Peripherals::take().unwrap();
        let dp = pac::Peripherals::take().unwrap();

        let pwr = dp.PWR.constrain();
        let pwrcfg = pwr.freeze();

        info!("Power enabled");
        // RCC
        let mut rcc = dp.RCC.constrain();
        let reset = rcc.get_reset_reason();

        info!("Reset reason: {:?}", reset);

        let ccdr = rcc
            .use_hse(48.MHz()) // check the clock hardware
            .sys_ck(200.MHz())
            .freeze(pwrcfg, &dp.SYSCFG);
        info!("RCC configured");
    }

    #[test]
    fn example_test() {
        assert!(true);
    }
}

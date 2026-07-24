use core::cell::RefCell;

use embassy_stm32::{gpio, mode, spi};
use embassy_time::Delay;
use embedded_hal_bus::spi::RefCellDevice;
use embedded_sdmmc::SdCard;

pub type SDCardSpiBus = spi::Spi<'static, mode::Blocking>; // Has to be blocking for embedded-sdmmc to work
pub type SDCardSpiRefCell = RefCell<SDCardSpiBus>;
pub type SDCardChipSelect = gpio::Output<'static>;
pub type SDCardSpiDevice = RefCellDevice<'static, SDCardSpiBus, SDCardChipSelect, Delay>;
pub type SDCardInstance = SdCard<SDCardSpiDevice, Delay>;

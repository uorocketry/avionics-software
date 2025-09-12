use embassy_stm32::{gpio, mode, spi};
use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::Delay;
use embedded_sdmmc::{Directory, SdCard, VolumeManager};
use heapless::String;
use core::cell::RefCell;
use embedded_hal_bus::spi::RefCellDevice;

use crate::sd::time_source::FakeTimeSource;

// Some typings to make the code more readable
pub type SDCardSpiBus = spi::Spi<'static, mode::Blocking>; // Has to be blocking for embedded-sdmmc to work
pub type SDCardSpiRefCell = RefCell<SDCardSpiBus>;
pub type SDCardChipSelect = gpio::Output<'static>;
pub type SDCardSpiDevice = RefCellDevice<'static, SDCardSpiBus, SDCardChipSelect, Delay>;

pub type SDCardInstance = SdCard<SDCardSpiDevice, Delay>;

pub type SDCardVolumeManager<const MAX_SESSIONS_COUNT: usize, const MAX_FILES_COUNT: usize> = VolumeManager <
	SDCardInstance,
	FakeTimeSource,
	MAX_SESSIONS_COUNT, // MAX_DIRS translates to MaxSessions count because each session is a directory
	MAX_FILES_COUNT,
	1
>;

pub type SDCardDirectory<'a, const MAX_SESSIONS_COUNT: usize, const MAX_FILES_COUNT: usize> = Directory<
	'a,
	SDCardInstance,
	FakeTimeSource,
	MAX_SESSIONS_COUNT, // MAX_DIRS translates to MaxSessions count because each session is a directory
	MAX_FILES_COUNT,
	1
>;

pub type FilePath = String<64>;
pub type Line = String<255>;

// Represents the scope of a read/write operation
#[derive(defmt::Format)]
pub enum OperationScope {
	Root, // Reads/Writes the file in the absolute path specified
	CurrentSession // Reads/Writes the file in the current session directory
}

pub type SDCardChannel<const CHANNEL_SIZE: usize> = Channel<CriticalSectionRawMutex, (OperationScope, FilePath, Line), CHANNEL_SIZE>;
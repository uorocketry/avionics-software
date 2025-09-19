use core::cell::RefCell;

use argus::sd::config::{MAX_LINE_LENGTH, QUEUE_SIZE};
use argus::sd::time_source::FakeTimeSource;
use embassy_stm32::{gpio, mode, spi};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Delay;
use embedded_hal_bus::spi::RefCellDevice;
use embedded_sdmmc::{Directory, SdCard, VolumeManager};
use heapless::String;

// Some typings to make the code more readable
pub type SDCardSpiBus = spi::Spi<'static, mode::Blocking>; // Has to be blocking for embedded-sdmmc to work
pub type SDCardSpiRefCell = RefCell<SDCardSpiBus>;
pub type SDCardChipSelect = gpio::Output<'static>;
pub type SDCardSpiDevice = RefCellDevice<'static, SDCardSpiBus, SDCardChipSelect, Delay>;

pub type SDCardInstance = SdCard<SDCardSpiDevice, Delay>;

pub type SDCardVolumeManager<const MAX_DIRS: usize, const MAX_FILES: usize> = VolumeManager<
	SDCardInstance,
	FakeTimeSource,
	MAX_DIRS, // MAX_DIRS translates to MaxSessions count because each session is a directory
	MAX_FILES,
	1,
>;

pub type SDCardDirectory<'a, const MAX_DIRS: usize, const MAX_FILES: usize> = Directory<
	'a,
	SDCardInstance,
	FakeTimeSource,
	MAX_DIRS, // MAX_DIRS translates to MaxSessions count because each session is a directory
	MAX_FILES,
	1,
>;

pub type FileName = String<12>; // FAT 8.3 format only allows 8 chars for name, 3 for extension and 1 for the dot
pub type DirectoryName = String<8>; // Max directory name length in FAT 8.3 is 8 characters
pub type Line = String<MAX_LINE_LENGTH>; // A line to be written to the SD card

// Represents the scope of a read/write operation
#[derive(defmt::Format)]
pub enum OperationScope {
	Root,           // Reads/Writes the file in the absolute path specified
	CurrentSession, // Reads/Writes the file in the current session directory
}

pub type SdOperationQueue = Channel<CriticalSectionRawMutex, (OperationScope, FileName, Line), QUEUE_SIZE>;

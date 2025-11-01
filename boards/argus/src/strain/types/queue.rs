use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

use crate::strain::config::STRAIN_READING_QUEUE_SIZE;
use crate::strain::types::strain_reading::StrainReading;

// Type alias for the strain reading queue used to decouple reading from ADC and writing to logging pipes
pub type StrainReadingQueue = Channel<CriticalSectionRawMutex, StrainReading, STRAIN_READING_QUEUE_SIZE>;

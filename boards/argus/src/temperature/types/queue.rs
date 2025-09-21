use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

use super::thermocouple_channel::ThermocoupleChannel;
use super::thermocouple_reading::ThermocoupleReading;
use crate::config::AdcDevice;

// Size of the queue used to send temperature readings from the temperature service to the SD card service
pub const QUEUE_SIZE: usize = 16;

// Type alias for the thermocouple reading queue used to decouple reading from ADC and writing to logging pipes
pub type ThermocoupleReadingQueue = Channel<CriticalSectionRawMutex, (AdcDevice, ThermocoupleChannel, ThermocoupleReading), QUEUE_SIZE>;

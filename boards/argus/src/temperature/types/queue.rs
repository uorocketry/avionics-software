use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

use super::thermocouple_channel::ThermocoupleChannel;
use super::thermocouple_reading::ThermocoupleReading;
use crate::adc::types::AdcDevice;
use crate::sd::config::QUEUE_SIZE;

// Type alias for the thermocouple reading queue used to decouple reading from ADC and writing to logging pipes
pub type ThermocoupleReadingQueue = Channel<CriticalSectionRawMutex, (AdcDevice, ThermocoupleChannel, ThermocoupleReading), QUEUE_SIZE>;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

use crate::adc::types::AdcDevice;
use crate::sd::config::QUEUE_SIZE;
use crate::temperature::types::thermocouple_channel::ThermocoupleChannel;
use crate::temperature::types::thermocouple_reading::ThermocoupleReading;

// Type alias for the thermocouple reading queue used to decouple reading from ADC and writing to logging pipes
pub type ThermocoupleReadingQueue = Channel<CriticalSectionRawMutex, (AdcDevice, ThermocoupleChannel, ThermocoupleReading), QUEUE_SIZE>;

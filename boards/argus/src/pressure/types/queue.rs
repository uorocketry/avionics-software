use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

use crate::adc::types::AdcDevice;
use crate::pressure::config::PRESSURE_READING_QUEUE_SIZE;
use crate::pressure::types::pressure_channel::PressureChannel;
use crate::pressure::types::pressure_reading::PressureReading;

// Type alias for the pressure reading queue used to decouple reading from ADC and writing to logging pipes
pub type PressureReadingQueue = Channel<CriticalSectionRawMutex, (AdcDevice, PressureChannel, PressureReading), PRESSURE_READING_QUEUE_SIZE>;

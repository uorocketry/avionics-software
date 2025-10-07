// Size of the queue used to send temperature readings from the temperature service to the SD card service
pub const TEMPERATURE_READING_QUEUE_SIZE: usize = 16;

// Maximum number of calibration data points allowed to be collected during a calibration session per thermocouple channel
pub const MAX_CALIBRATION_DATA_POINTS: usize = 10;

// File name used to read/write linear transformations that applied to thermocouple readings to/from the SD card
// Linear transformations are stored in CSV format
pub const LINEAR_TRANSFORMATIONS_FILE_NAME: &str = "t_temp.csv"; // Cannot be longer than 12 characters

// Resistance of the RTD at 0 °C.
pub const RTD_RESISTANCE_AT_0C: f32 = 1000.0; // Ohms

// Measure RTDs at a slower interval than the thermocouples
pub const RTD_MEASUREMENT_INTERVAL: u64 = 5000; // milliseconds

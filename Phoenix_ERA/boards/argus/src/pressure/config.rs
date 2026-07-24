// Size of the queue used to send pressure readings from the pressure service to the SD card service
pub const PRESSURE_READING_QUEUE_SIZE: usize = 16;

// Maximum number of calibration data points allowed to be collected during a calibration session per pressure channel
pub const MAX_CALIBRATION_DATA_POINTS: usize = 10;

// File name used to read/write linear transformations that applied to pressure readings to/from the SD card
// Linear transformations are stored in CSV format
pub const LINEAR_TRANSFORMATIONS_FILE_NAME: &str = "t_pres.csv"; // Cannot be longer than 12 characters

// Resistance of the NTC at 25 °C.
pub const NTC_RESISTANCE_AT_25C: f32 = 10000.0; // Ohms

// Measure NTCs at a slower interval than the pressures
pub const NTC_MEASUREMENT_INTERVAL: u64 = 5000; // milliseconds

// Number of pressure sensor channels per ADC
// Note: Not to get confused with the number of analog input channels on each ADC
// Each pressure sensor channel uses a pair of analog input channels (differential measurement)
pub const PRESSURE_CHANNEL_COUNT: usize = 4;

// Size of the queue used to send pressure readings from the pressure service to the SD card service
pub const PRESSURE_READING_QUEUE_SIZE: usize = 16;

// Maximum number of calibration data points allowed to be collected during a calibration session per thermocouple channel
pub const MAX_CALIBRATION_DATA_POINTS: usize = 10;

// File name used to read/write linear transformations that applied to thermocouple readings to/from the SD card
// Linear transformations are stored in CSV format
pub const LINEAR_TRANSFORMATIONS_FILE_NAME: &str = "t_pres.csv"; // Cannot be longer than 12 characters

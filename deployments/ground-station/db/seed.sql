--#region RGS tables

-- Table to store commands requested by the ground station to be sent to the rocket.
CREATE TABLE IF NOT EXISTS OutgoingCommand (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    command_type TEXT NOT NULL, -- e.g., 'DeployDrogue', 'Online', 'Ping'
    parameters TEXT, -- JSON string or simple value representing command arguments. Can be NULL if no params.
    status TEXT NOT NULL, -- e.g., 'Pending', 'Queued', 'Sending', 'Sent', 'Failed', 'Cancelled'
    created_at INTEGER NOT NULL, -- UNIX epoch timestamp when the command was requested
    queued_at INTEGER, -- UNIX epoch timestamp when the dispatcher picked it up (optional)
    sent_at INTEGER, -- UNIX epoch timestamp when the dispatcher attempted to send it (optional)
    attempts INTEGER NOT NULL DEFAULT 0, -- Number of send attempts
    error_message TEXT, -- Details if sending failed (optional)
    source_service TEXT NOT NULL -- e.g., 'rgs-web', 'command-dispatcher-keepalive'
);

-- Index for efficient retrieval of pending commands by the dispatcher
CREATE INDEX IF NOT EXISTS idx_outgoingcommand_status_created ON OutgoingCommand (status, created_at);

-- Table to store system-wide configuration settings.
CREATE TABLE IF NOT EXISTS SystemConfig (
    key TEXT PRIMARY KEY, -- Unique configuration key, e.g., 'gateway_mode', 'serial_port_path'
    value TEXT NOT NULL, -- The configuration value (stored as text)
    description TEXT, -- Optional description of the setting
    updated_at INTEGER NOT NULL -- UNIX epoch timestamp when the setting was last updated
);


-- Table to store the current status of each running service instance.
CREATE TABLE IF NOT EXISTS ServiceStatus (
    service_instance_id TEXT PRIMARY KEY, -- Unique ID for a service instance (e.g., 'rgs-telemetry-ingestor@hostname-pid')
    service_name TEXT NOT NULL, -- e.g., 'rgs-telemetry-ingestor', 'rgs-web'
    hostname TEXT, -- Hostname where the service is running (optional)
    status TEXT NOT NULL, -- e.g., 'Running', 'Degraded', 'Error', 'Stopped'
    status_message TEXT, -- Short message detailing current state (optional)
    last_heartbeat_at INTEGER NOT NULL, -- UNIX epoch timestamp of the last status update
    start_time INTEGER -- UNIX epoch timestamp when the service started (optional)
);

-- Index for efficient retrieval of service statuses by name
CREATE INDEX IF NOT EXISTS idx_servicestatus_name ON ServiceStatus (service_name);

CREATE TABLE IF NOT EXISTS ServicePing (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    service_id TEXT NOT NULL,
    hostname TEXT,
    app_timestamp INTEGER NOT NULL,
    db_timestamp INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE INDEX IF NOT EXISTS idx_serviceping_db_timestamp ON ServicePing (db_timestamp);
CREATE INDEX IF NOT EXISTS idx_serviceping_service_db_timestamp ON ServicePing (service_id, db_timestamp);

--#endregion

--#region Incoming telemetry tables
CREATE TABLE IF NOT EXISTS RadioFrame (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL, -- ISO 8601 format
    timestamp_epoch INTEGER NOT NULL, -- UNIX epoch time for range queries
    node TEXT NOT NULL, -- Enum Node type (PressureBoard, StrainBoard, etc)
    data_type TEXT NOT NULL, -- Payload type: "Command" | "Log" | "PhoenixState" | "PhoenixEvent" | "ArgusState" | "ArgusEvent" | "SbgGpsPos" | "SbgUtcTime" | "SbgImu" | "SbgEkfQuat" | "SbgEkfNav" | "SbgGpsVel" | "SbgAir" | "Gps" | "Imu" | "Madgwick" | "Barometer" | "ArgusPressure" | "ArgusTemperature" | "ArgusStrain"
    data_id INTEGER NOT NULL, -- Foreign key to specific data table
    millis_since_start INTEGER -- Milliseconds since node start, if provided
);


CREATE TABLE IF NOT EXISTS Command (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    data_type TEXT NOT NULL, -- Enum Command subtype (e.g., "DeployDrogue", "DeployMain", "PowerDown", "RadioRateChange", "Online")
    data_id INTEGER -- Foreign key to subtype-specific table
);


--#region Command subtypes
CREATE TABLE IF NOT EXISTS Online (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    online BOOLEAN NOT NULL
);

CREATE TABLE IF NOT EXISTS DeployDrogue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    val BOOLEAN NOT NULL
);

CREATE TABLE IF NOT EXISTS DeployMain (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    val BOOLEAN NOT NULL
);

CREATE TABLE IF NOT EXISTS PowerDown (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    board TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS RadioRateChange (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    rate TEXT NOT NULL
);

-- Camera power control commands (no additional fields for now)
CREATE TABLE IF NOT EXISTS PowerUpCamera (
    id INTEGER PRIMARY KEY AUTOINCREMENT
);

CREATE TABLE IF NOT EXISTS PowerDownCamera (
    id INTEGER PRIMARY KEY AUTOINCREMENT
);

CREATE TABLE IF NOT EXISTS Ping (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ping_id INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS Pong (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pong_id INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS Log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    level TEXT NOT NULL,
    event TEXT NOT NULL,
    message BLOB
);

CREATE TABLE IF NOT EXISTS State (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    state TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS Event (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event TEXT NOT NULL
);


--#endregion

--#region Sbg

CREATE TABLE IF NOT EXISTS SbgUtcTime (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    time_stamp INTEGER NOT NULL,
    status TEXT NOT NULL,
    year INTEGER,
    month INTEGER,
    day INTEGER,
    hour INTEGER,
    minute INTEGER,
    second INTEGER,
    nano_second INTEGER,
    gps_time_of_week INTEGER
);

CREATE TABLE IF NOT EXISTS SbgAir (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    time_stamp INTEGER NOT NULL,
    status TEXT NOT NULL,
    pressure_abs REAL,
    altitude REAL,
    pressure_diff REAL,
    true_airspeed REAL,
    air_temperature REAL
);

CREATE TABLE IF NOT EXISTS SbgEkfQuat (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    time_stamp INTEGER NOT NULL,
    quaternion_w REAL,
    quaternion_x REAL,
    quaternion_y REAL,
    quaternion_z REAL,
    euler_std_dev_roll REAL,
    euler_std_dev_pitch REAL,
    euler_std_dev_yaw REAL,
    status TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS SbgEkfNav (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    status TEXT NOT NULL,
    velocity_north REAL,
    velocity_east REAL,
    velocity_down REAL,
    velocity_std_dev_north REAL,
    velocity_std_dev_east REAL,
    velocity_std_dev_down REAL,
    position_latitude REAL,
    position_longitude REAL,
    position_altitude REAL,
    position_std_dev_latitude REAL,
    position_std_dev_longitude REAL,
    position_std_dev_altitude REAL,
    time_stamp INTEGER NOT NULL,
    undulation REAL
);

CREATE TABLE IF NOT EXISTS SbgImu (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    time_stamp INTEGER NOT NULL,
    status TEXT NOT NULL,
    accelerometer_x REAL,
    accelerometer_y REAL,
    accelerometer_z REAL,
    gyroscope_x REAL,
    gyroscope_y REAL,
    gyroscope_z REAL,
    delta_velocity_x REAL,
    delta_velocity_y REAL,
    delta_velocity_z REAL,
    delta_angle_x REAL,
    delta_angle_y REAL,
    delta_angle_z REAL,
    temperature REAL
);

CREATE TABLE IF NOT EXISTS SbgGpsVel (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    time_of_week INTEGER,
    time_stamp INTEGER NOT NULL,
    status TEXT NOT NULL,
    velocity_north REAL,
    velocity_east REAL,
    velocity_down REAL,
    velocity_acc_north REAL,
    velocity_acc_east REAL,
    velocity_acc_down REAL,
    course REAL,
    course_acc REAL
);

CREATE TABLE IF NOT EXISTS SbgGpsPos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    latitude REAL,
    longitude REAL,
    time_of_week INTEGER,
    undulation REAL,
    altitude REAL,
    time_stamp INTEGER NOT NULL,
    status TEXT NOT NULL,
    latitude_accuracy REAL,
    longitude_accuracy REAL,
    altitude_accuracy REAL,
    num_sv_used INTEGER,
    base_station_id INTEGER,
    differential_age INTEGER
);

-- Table to store radio metrics such as RSSI and packet loss statistics
CREATE TABLE IF NOT EXISTS RadioMetrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    rssi INTEGER,
    packets_lost INTEGER
);

CREATE INDEX IF NOT EXISTS idx_radiometrics_timestamp ON RadioMetrics (timestamp);

-- Summary table to maintain aggregates for fast queries
CREATE TABLE IF NOT EXISTS MetricsSummary (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    total_packets_lost INTEGER NOT NULL DEFAULT 0,
    total_radio_frames INTEGER NOT NULL DEFAULT 0
);

-- Ensure a single row exists
INSERT INTO MetricsSummary (id, total_packets_lost, total_radio_frames)
    SELECT 1, 0, 0
    WHERE NOT EXISTS (SELECT 1 FROM MetricsSummary WHERE id = 1);

-- Triggers to update summary on inserts
CREATE TRIGGER IF NOT EXISTS trg_radiometrics_insert
AFTER INSERT ON RadioMetrics
BEGIN
    UPDATE MetricsSummary
    SET total_packets_lost = total_packets_lost + COALESCE(NEW.packets_lost, 0);
END;

CREATE TRIGGER IF NOT EXISTS trg_radioframe_insert
AFTER INSERT ON RadioFrame
BEGIN
    UPDATE MetricsSummary
    SET total_radio_frames = total_radio_frames + 1;
END;

-- Sensor tables for protobuf messages
CREATE TABLE IF NOT EXISTS Iim20670Imu (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp_us INTEGER NOT NULL,
    gyro_x REAL,
    gyro_y REAL,
    gyro_z REAL,
    accel_x REAL,
    accel_y REAL,
    accel_z REAL,
    temperature_1 INTEGER,
    temperature_2 INTEGER,
    accel_fs INTEGER,
    gyro_fs INTEGER,
    spi_status INTEGER,
    low_res_accel_x REAL,
    low_res_accel_y REAL,
    low_res_accel_z REAL
);

CREATE TABLE IF NOT EXISTS Madgwick (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    quat_w REAL,
    quat_x REAL,
    quat_y REAL,
    quat_z REAL
);

-- Barometer message table (MS5611)
CREATE TABLE IF NOT EXISTS Barometer (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pressure_kpa REAL,
    temperature_celsius REAL
);

-- Phoenix state/event tables
CREATE TABLE IF NOT EXISTS PhoenixState (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    state TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS PhoenixEvent (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event TEXT NOT NULL
);

-- Argus state/event tables
CREATE TABLE IF NOT EXISTS ArgusState (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    state TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS ArgusEvent (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event TEXT NOT NULL
);

-- Argus sensor tables
CREATE TABLE IF NOT EXISTS ArgusPressure (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pressure REAL,
    sensor_id INTEGER
);

CREATE TABLE IF NOT EXISTS ArgusTemperature (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    temperature REAL,
    sensor_id INTEGER
);

CREATE TABLE IF NOT EXISTS ArgusStrain (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    strain REAL,
    sensor_id INTEGER
);


--#endregion

--#endregion

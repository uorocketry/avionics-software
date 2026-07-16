# GPS Ingest Architecture

## Overview

The optional antenna-tracker toolset is a focused data-processing pipeline rather than a deployed RGS microservice. Its `gps-ingest` binary streams GPS coordinates and altitude data from MAVLink POSTCARD messages as JSON lines for the Python predictor and antenna-control tools.

## System Architecture

This program is designed to fit into the antenna tracker architecture shown in the system diagram:

```
MAVLink Source → gps-ingest → JSON Lines → Predictor (Python) → Antenna Tracker API
     ↓                ↓            ↓              ↓                    ↓
  SerGW/Radio    (Rust)      stdout pipe    example_pipeline.py    (External System)
```

### Component Roles

1. **GPS Stream (gps-ingest)**
   - **Language**: Rust
   - **Input**: MAVLink POSTCARD messages (via TCP/UDP connection)
   - **Output**: JSON lines to stdout
   - **Purpose**: Extract and format GPS/altitude data from MAVLink telemetry

2. **Predictor (example_pipeline.py)**
   - **Language**: Python
   - **Input**: JSON lines from gps-ingest (via pipe)
   - **Output**: Prediction commands
   - **Purpose**: Process GPS data and predict future positions

3. **Antenna Tracker API**
   - **Language**: Python library
   - **Input**: Commands from predictor
   - **Output**: Hardware control signals
   - **Purpose**: Control antenna positioning hardware

## Data Flow

```
MAVLink POSTCARD → RadioFrame (protobuf) → SbgGpsPos/SbgAir → JSON Lines → Python Script → Predictions → API Commands
```

### MAVLink Message Processing

The program processes the following MAVLink messages:

- **POSTCARD_MESSAGE**: Contains a protobuf-encoded `RadioFrame` with telemetry data
  - Extracts `SbgGpsPos` data for GPS coordinates (latitude, longitude)
  - Extracts `SbgAir` data for barometric altitude
- **RADIO_STATUS**: Logged for monitoring but not used for GPS output

### Data Validation

Before outputting a sample, the program validates:
- GPS coordinates are not near (0, 0) - invalid GPS fix
- GPS coordinates are within Canada bounds (lat: 41.68 to 83.11, lon: -141.00 to -52.62)
- All required fields are present (lat, lon, altitude_m)

### JSON Output Format

The program outputs one JSON line per complete sample:

```json
{"lat":45.123456,"lon":-75.654321,"altitude_m":100.5,"ts":"2024-01-01T12:00:00Z","source":"gps-ingest"}
```

Each line contains:
- `lat`: Latitude in degrees (f64)
- `lon`: Longitude in degrees (f64)
- `altitude_m`: Altitude in meters (f64) - from barometric pressure, not GPS
- `ts`: ISO 8601 timestamp (RFC3339 format)
- `source`: Always "gps-ingest"

The program only outputs when it has received both GPS position and altitude data, ensuring complete samples.

## Usage Patterns

### Basic Usage
```bash
# Stream GPS data (default: tcpout:127.0.0.1:5656)
cargo run -p gps-ingest

# Use different MAVLink connection
cargo run -p gps-ingest -- --connection udpin:0.0.0.0:14550

# Pretty JSON output
cargo run -p gps-ingest -- --pretty
```

### Piping to Other Programs
```bash
# Extract just latitude
cargo run -p gps-ingest | jq -r '.lat'

# Format coordinates
cargo run -p gps-ingest | jq -r '"\(.lat),\(.lon),\(.altitude_m)"'

# Filter recent samples
cargo run -p gps-ingest | jq -r 'select(.ts | fromdateiso8601 > (now - 3600))'
```

### Integration with Python
```python
import subprocess
import json

# Run gps-ingest and capture output line by line
proc = subprocess.Popen(
    ["cargo", "run", "-p", "gps-ingest"],
    stdout=subprocess.PIPE,
    text=True
)

for line in proc.stdout:
    line = line.strip()
    if not line:
        continue
    try:
        data = json.loads(line)
        print(f"Position: {data['lat']}, {data['lon']}, Altitude: {data['altitude_m']}m")
    except json.JSONDecodeError:
        continue
```

## Design Principles

1. **Single Responsibility**: Only extracts GPS data from MAVLink, doesn't process it
2. **Unix Philosophy**: Does one thing well, outputs to stdout
3. **JSON Lines**: One complete sample per line for easy streaming and parsing
4. **Configurable**: MAVLink connection string can be specified
5. **Error Handling**: Automatic reconnection on connection loss
6. **Data Validation**: Filters invalid coordinates before output
7. **Real-time**: Streams data as it arrives, not batch processing

## Connection Management

- Automatically connects to the specified MAVLink endpoint
- Handles connection failures gracefully with automatic reconnection
- Reconnection delay: 1 second after connection loss
- Logs connection status and errors using structured logging (tracing)

## Future Enhancements

- **Connection Retry Backoff**: Exponential backoff for reconnection attempts
- **Data Filtering**: Time range and quality filters
- **Multiple Formats**: CSV, XML, or binary output options
- **WebSocket Output**: Real-time streaming over network
- **Data Validation**: Additional quality checks and outlier detection
- **Rate Limiting**: Configurable output rate limiting

## Dependencies

- **mavlink**: MAVLink protocol support for uORocketry message set
- **messages-prost**: Protobuf message definitions for RadioFrame and SBG data
- **serde**: JSON serialization/deserialization
- **tokio**: Async runtime for MAVLink I/O operations
- **clap**: Command-line argument parsing
- **chrono**: Timestamp handling and formatting
- **tracing**: Structured logging
- **prost**: Protobuf decoding
- **anyhow**: Error handling

## Building and Testing

```bash
# Build the project
cargo build --release

# Check compilation
cargo check

# Run with verbose logging
RUST_LOG=debug cargo run
```

## Integration Notes

- The program is designed to be lightweight and fast
- JSON lines output is optimized for parsing by other tools
- Error conditions are handled gracefully with automatic reconnection
- The program can be easily integrated into CI/CD pipelines
- MAVLink connections are properly managed and automatically recovered
- Designed to work with SerGW (Serial Gateway) which bridges serial devices to TCP

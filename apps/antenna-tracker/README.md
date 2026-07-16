# Antenna Tracker Tools

Optional antenna-tracking tools, including the Rust `gps-ingest` stream extractor and Python prediction/control pipeline. This directory is a development and field utility, not a default RGS runtime service.

## Features

- Connects to MAVLink and listens for POSTCARD messages
- Extracts GPS position data from `SbgGpsPos` messages
- Extracts altitude data from `SbgAir` messages
- Validates coordinates (filters invalid coordinates near 0,0 and outside Canada bounds)
- Outputs JSON lines to stdout (one complete sample per line)
- Automatic reconnection on connection loss
- Designed to be piped to other programs

## Usage

### Basic usage (default MAVLink connection)
```bash
cargo run -p gps-ingest
```

### Specify MAVLink connection string
```bash
cargo run -p gps-ingest -- --connection tcpout:127.0.0.1:5656
```

### Use UDP input instead
```bash
cargo run -- --connection udpin:0.0.0.0:14550
```

### Pretty JSON output (multi-line)
```bash
cargo run -p gps-ingest -- --pretty
```

### Pipe output to another program
```bash
cargo run | jq '.lat'
```

## Command Line Options

- `--connection`: MAVLink connection string (default: `tcpout:127.0.0.1:5656`)
  - Examples: `tcpout:127.0.0.1:5656`, `udpin:0.0.0.0:14550`, `udpout:192.168.1.100:14550`
- `--pretty`: Print pretty JSON (multi-line) instead of compact single-line JSON (default: false)
- `-h, --help`: Show help information

## Output Format

The program outputs JSON lines (one per complete sample) with the following structure:

```json
{"lat":45.123456,"lon":-75.654321,"altitude_m":100.5,"ts":"2024-01-01T12:00:00Z","source":"gps-ingest"}
```

Each line contains:
- `lat`: Latitude in degrees (f64)
- `lon`: Longitude in degrees (f64)
- `altitude_m`: Altitude in meters (f64)
- `ts`: ISO 8601 timestamp (RFC3339 format)
- `source`: Always "gps-ingest"

The program only outputs a line when it has received both GPS position (from `SbgGpsPos`) and altitude (from `SbgAir`) data, ensuring complete samples.

## How It Works

1. Connects to the specified MAVLink endpoint
2. Listens for `POSTCARD_MESSAGE` MAVLink messages
3. Decodes the protobuf `RadioFrame` from each POSTCARD message
4. Extracts GPS coordinates from `SbgGpsPos` data
5. Extracts altitude from `SbgAir` data
6. Validates coordinates (must be within Canada bounds and not near 0,0)
7. Outputs a JSON line when all required fields are available
8. Automatically reconnects if the connection is lost

## Building

```bash
cargo build --release
```

The binary will be available at `target/release/gps-ingest`.

## Dependencies

- `mavlink`: MAVLink protocol support
- `messages-prost`: Protobuf message definitions for radio frames
- `serde` + `serde_json`: JSON serialization
- `chrono`: Timestamp handling
- `tokio`: Async runtime
- `clap`: Command line argument parsing
- `tracing`: Structured logging
- `anyhow`: Error handling
- `prost`: Protobuf decoding

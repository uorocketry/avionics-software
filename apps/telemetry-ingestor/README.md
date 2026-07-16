# Telemetry Ingestor

A service that ingests telemetry data (MAVLink messages, specifically `POSTCARD_MESSAGE` and `RADIO_STATUS`), processes it, and stores it in a LibSQL database. It includes features like batch processing for efficiency and a heartbeat mechanism for service monitoring.

## Features

- Decodes MAVLink messages (specifically `POSTCARD_MESSAGE` and `RADIO_STATUS`)
- Stores telemetry data in a LibSQL database.
- Batch processes messages for improved database write performance.
- Monitors service health with an automatic heartbeat to the database.

## Prerequisites

- LibSQL server up and running
- Network access to the MAVLink message source (e.g., SerGW)

## Usage

### Running the Ingestor

```sh
cargo run -- [options]
# Or directly if built:
# ./target/debug/telemetry-ingestor [options]
```

### Parameters

| Parameter                   | Description                                                              | Default                    |
|-----------------------------|--------------------------------------------------------------------------|----------------------------|
| `--libsql-url`              | LibSQL server URL                                                      | http://localhost:8080      |
| `--gateway-connection-string` | Gateway MAVLink connection string (e.g., `tcpout:localhost:5656`)        | `tcpout:127.0.0.1:5656`    |

## Operational Details

- Messages are batched for database insertion, with a batch size of 100 messages or a 500ms timeout.
- A heartbeat is sent to the database every 30 seconds.
- Currently processes `POSTCARD_MESSAGE` (saving content) and logs `RADIO_STATUS` (saving not yet implemented).
- Tracks MAVLink packet sequence numbers and logs detected packet loss.

## Troubleshooting

1. **Database Issues**
   - Verify LibSQL server is running and accessible.
   - Check that the `--libsql-url` parameter is correct.

2. **Connection Issues**
   - Verify the MAVLink source (e.g., SerGW) is running and accessible.
   - Check that `--gateway-connection-string` parameter is correct.
   - Ensure network connectivity between the ingestor and the MAVLink source.

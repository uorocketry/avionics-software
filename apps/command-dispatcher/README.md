# RGS Command Dispatcher

A service that monitors the `OutgoingCommand` table in the RGS database for pending commands and sends them to the designated gateway via a TCP MAVLink connection. It also reports its own health status.

## Features

- Polls the `OutgoingCommand` table for commands with status 'Pending'.
- Parses command types and parameters from the database.
- Constructs appropriate `messages::RadioMessage` payloads.
- Serializes messages using `postcard`.
- Sends MAVLink `POSTCARD_MESSAGE` packets to a TCP gateway.
- Updates command status (`Sending`, `Sent`, `Failed`) in the database.
- Manages connection to the TCP gateway, attempting reconnection on failure.
- Reports its operational status to the `ServiceStatus` table.
- Queues a periodic Ping keepalive, replacing the former standalone heartbeat service.

## Prerequisites

- RGS LibSQL database (with `OutgoingCommand` and `ServiceStatus` tables) accessible.
- RGS Gateway service running and accepting TCP MAVLink connections.

## Usage

### Running the Dispatcher

```sh
# From the rgs workspace root
cargo run -p command-dispatcher -- [options]
# Or directly if built:
# ./target/debug/command-dispatcher [options]
```

### Parameters

| Parameter                   | Description                                                              | Default                    |
|-----------------------------|--------------------------------------------------------------------------|----------------------------|
| `--libsql-url`              | LibSQL database URL                                                      | http://localhost:8080      |
| `--libsql-auth-token`       | Auth token for remote LibSQL DB                                            | *empty*                    |
| `--gateway-connection-string` | MAVLink gateway connection string (e.g., `tcpout:localhost:5656`)        | `tcpout:127.0.0.1:5656`    |
| `--poll-interval-secs`      | DB polling interval (seconds)                                            | 5                          |
| `--keepalive-interval-secs` | Automatic Ping interval; set to `0` to disable                           | 30                         |

## Operational Details

- Checks the database for pending commands every `--poll-interval-secs` seconds.
- Attempts to maintain a persistent connection to the gateway, reconnecting if the connection drops.
- Processes commands sequentially in the order they are fetched (ordered by `created_at`).
- Updates its status in the `ServiceStatus` table every 15 seconds.
- Keeps at most one dispatcher-generated Ping pending or sending, preventing backlog growth while the gateway is unavailable.
- Handles command types: `Ping`, `DeployDrogue`, `DeployMain`, `PowerDown`, `RadioRateChange`.

## Troubleshooting

1.  **Database Issues**
    - Verify LibSQL server is running and accessible.
    - Check `--libsql-url` and `--libsql-auth-token` parameters.
    - Ensure the `OutgoingCommand` and `ServiceStatus` tables exist.
2.  **Gateway Connection Issues**
    - Verify the Gateway service is running and accessible.
    - Check `--gateway-connection-string` parameter.
    - Ensure network connectivity between the dispatcher and the gateway.
    - Check dispatcher logs for connection attempt errors.
3.  **Commands Stuck in 'Pending' or 'Sending'**
    - Check dispatcher logs for errors during command processing (parsing, serialization, sending).
    - Verify the gateway is correctly receiving and acknowledging messages (if applicable).
    - Check the command `attempts` count and `error_message` in the `OutgoingCommand` table.

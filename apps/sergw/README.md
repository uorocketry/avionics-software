# SERGW - Serial Gateway

## NAME

sergw - Serial-to-TCP gateway for communication between serial devices and TCP clients

## SYNOPSIS

**sergw** [*OPTIONS*] _COMMAND_

**sergw ports**

**sergw listen** [*OPTIONS*] --serial _PORT_

## DESCRIPTION

SerGW is a serial-to-TCP gateway that bridges communication between serial devices and TCP network clients. It allows multiple TCP connections to communicate with a single serial device.

The gateway uses multiple threads to handle serial port I/O and TCP connections. Data from the serial port is broadcast to all connected TCP clients, while data from any TCP client is forwarded to the serial port.

## COMMANDS

### ports

Lists all available USB serial ports on the system. Outputs port names separated by spaces, suitable for scripting.

**Example:**

```bash
sergw ports
# Output: /dev/ttyUSB0 /dev/ttyUSB1 /dev/ttyACM0
```

### listen

Starts the TCP server and establishes serial communication.

**Required Options:**

- `--serial *PORT*` - Path to the serial port device (e.g., `/dev/ttyUSB0`, `COM3`)

**Optional Options:**

- `--baud *RATE*` - Baud rate for serial communication (default: 57600)
- `--host *ADDRESS*` - TCP host and port to bind to (default: 127.0.0.1:5656)
- `-v, --verbose` - Enable verbose output showing detailed data transmission information

## OPTIONS

| Option          | Description                    | Default        |
| --------------- | ------------------------------ | -------------- |
| `--serial`      | Serial port device path        | Required       |
| `--baud`        | Serial communication baud rate | 57600          |
| `--host`        | TCP bind address and port      | 127.0.0.1:5656 |
| `-v, --verbose` | Enable verbose logging         | false          |

## EXAMPLES

**Basic usage:**

```bash
sergw listen --serial /dev/ttyUSB0
```

**Custom configuration:**

```bash
sergw listen --serial /dev/ttyUSB0 --baud 115200 --host 0.0.0.0:8080
```

**Verbose mode:**

```bash
sergw listen --serial /dev/ttyUSB0 --verbose
```

**List available ports:**

```bash
sergw ports
```

## ARCHITECTURE

SerGW uses multiple threads with the following components:

1. **Serial Reader Thread** - Continuously reads data from the serial port and broadcasts to all TCP clients
2. **TCP Listener Thread** - Accepts new TCP connections and spawns handler threads
3. **TCP Handler Threads** - One per client connection, handles bidirectional data forwarding
4. **Shared State Management** - Thread-safe coordination between serial and TCP operations

## ERROR HANDLING

SerGW handles errors for:

- **Serial Port Errors**: Connection failures, permission denied, device busy
- **TCP Network Errors**: Bind failures, connection timeouts, network unreachable
- **Data Transmission Errors**: Write failures, broken pipes, connection drops
- **Resource Management**: Cleanup of failed connections and resources

Error messages are written to stderr, while operational information is written to stdout.

## REQUIREMENTS

- **Rust Toolchain**: Install via [rustup](https://rustup.rs/)
- **Serial Port Access**: User must have permissions to access serial devices
  - Linux: Add user to `dialout` group
  - Windows: Administrator privileges may be required

## TROUBLESHOOTING

### Permission Denied on Serial Port

**Linux:**

```bash
sudo usermod -a -G dialout $USER
# Log out and back in, or use newgrp dialout
```

**Windows:**

- Run as Administrator or ensure proper COM port permissions

### Serial Port Not Found

1. Verify device is connected and powered
2. Check if port is in use by another process:
   ```bash
   lsof /dev/ttyUSB0  # Linux
   ```
3. Try different baud rates if communication fails

### TCP Connection Issues

1. **Port Already in Use:**
   ```bash
   netstat -tulpn | grep :5656  # Linux
   ```
2. **Firewall Blocking:**
   - Ensure firewall allows incoming connections on specified port
3. **Network Interface Binding:**
   - Use `0.0.0.0` instead of `127.0.0.1` for network access

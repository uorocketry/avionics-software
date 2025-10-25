# SERGW - Serial Gateway

## NAME

sergw - Serial-to-TCP gateway for communication between serial devices and TCP clients

## SYNOPSIS

**sergw** [*OPTIONS*] _COMMAND_

**sergw ports**

**sergw listen** [*OPTIONS*] --serial _PORT_

## DESCRIPTION

sergw bridges communication between a serial device and multiple TCP clients. Data from the serial port is broadcast to all connected TCP clients. Data from any TCP client is forwarded to the serial port.

The gateway automatically reconnects to the serial port if the connection is lost. Graceful shutdown is supported via Ctrl-C.

## COMMANDS

### ports

Lists available USB serial ports. Outputs port names separated by spaces.

```bash
sergw ports
# Output: /dev/ttyUSB0 /dev/ttyUSB1 /dev/ttyACM0
```

### listen

Starts the TCP server and establishes serial communication.

**Required Options:**

- `--serial *PORT*` - Serial port device path (e.g., `/dev/ttyUSB0`, `COM3`)

**Optional Options:**

- `--baud *RATE*` - Serial baud rate (default: 57600)
- `--host *ADDRESS*` - TCP bind address and port (default: 127.0.0.1:5656)
- `-v, --verbose` - Enable verbose output

## OPTIONS

| Option          | Description               | Default        |
| --------------- | ------------------------- | -------------- |
| `--serial`      | Serial port device path   | Required       |
| `--baud`        | Serial baud rate          | 57600          |
| `--host`        | TCP bind address and port | 127.0.0.1:5656 |
| `-v, --verbose` | Enable verbose logging    | false          |

## EXAMPLES

```bash
sergw listen --serial /dev/ttyUSB0
sergw listen --serial /dev/ttyUSB0 --baud 115200 --host 0.0.0.0:8080
sergw listen --serial /dev/ttyUSB0 --verbose
sergw ports
```

## ARCHITECTURE

The gateway uses multiple threads:

1. **Main Reconnect Loop** - Attempts to connect to the serial port and spawns a reader thread. If the connection is lost, it cleans up and retries after a delay.

2. **Serial Reader Thread** - Reads data from the serial port and broadcasts to all TCP clients.

3. **Serial Writer Thread** - Receives data from TCP clients and writes to the serial port.

4. **TCP Listener Thread** - Accepts new TCP connections and spawns handler threads.

5. **TCP Client Handler Threads** - One per client, handles bidirectional data forwarding.

Thread coordination uses shared state (`Arc<Mutex<...>>`), message channels (`std::sync::mpsc`), and an atomic shutdown flag.

## REQUIREMENTS

- Rust toolchain (install via [rustup](https://rustup.rs/))
- Serial port access permissions
  - Linux: Add user to `dialout` group
  - Windows: Administrator privileges may be required

## TROUBLESHOOTING

### Permission Denied on Serial Port

**Linux:**

```bash
sudo usermod -a -G dialout $USER
# Log out and back in
```

**Windows:**

- Run as Administrator

### Serial Port Not Found

1. Verify device is connected and powered
2. Check if port is in use:
   ```bash
   lsof /dev/ttyUSB0  # Linux
   ```
3. Try different baud rates

### TCP Connection Issues

1. **Port in use:**
   ```bash
   netstat -tulpn | grep :5656  # Linux
   ```
2. **Firewall blocking:** Allow incoming connections on specified port
3. **Network access:** Use `0.0.0.0` instead of `127.0.0.1`

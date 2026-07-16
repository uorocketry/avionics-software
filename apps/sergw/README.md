### sergw

Simple serial ↔ TCP gateway with a built‑in TUI and optional zero‑config mDNS advertisement.

sergw opens a local serial device and serves it over TCP, broadcasting serial output to all connected clients and forwarding client input to the serial device. It emphasizes pragmatic reliability (auto‑reconnect) and visibility (TUI overview and hex/ascii/dec inspector).

### Highlights

- Serial ↔ TCP bridge over raw TCP (no telnet/RFC2217, no TLS)
- Multi‑client fan‑out: serial output is broadcast to all connected TCP clients
- Backpressure handling: slow or disconnected clients are dropped
- TUI: overview (connections, throughput, events) and inspector (hex/ascii/dec)
- Auto‑reconnect for serial (reader/writer) with buffered retry for writes
- Zero‑config mDNS (feature ‘mdns’, enabled by default): `_sergw._tcp`
- Linux mock tools: PTY‑backed mock serial and TCP chat helper (Linux only)

### Install

- From crates.io (default features include mDNS):

```
cargo install sergw
```

- Without mDNS:

```
cargo install sergw --no-default-features
```

### Quick start

Bridge a serial device on port 5656 (defaults shown):

```
sergw listen --serial /dev/ttyUSB0 --baud 115200 --host 127.0.0.1:5656
```

Connect a TCP client (e.g. `nc 127.0.0.1 5656`) to interact.

### CLI

```
sergw
  ports [--all] [--verbose] [--format text|json]
  listen [--serial <PATH>] [--baud <u32>] [--host <addr:port>]
         [--data-bits five|six|seven|eight]
         [--parity none|odd|even]
         [--stop-bits one|two]
         [--buffer <usize>]
  mock serial [--alias <PATH>]           # Linux only
  mock listener [--host <addr:port>]     # Linux only
```

- `ports`: list serial ports (USB‑only by default). Use `--all` to include non‑USB. `--format json` for machine output.
- `listen`: start the bridge. If `--serial` is omitted and exactly one USB serial is present, it is auto‑selected; otherwise a helpful error is returned.
- `mock serial` (Linux): create a PTY that behaves like a serial device and open a TUI to interact.
- `mock listener` (Linux): connect to a TCP server with a TUI (handy for testing the bridge from the client side).

### mDNS / Bonjour (optional)

When built with the `mdns` feature (default), sergw advertises the service using type `_sergw._tcp`.

- Instance name: `sergw:<ttyname>` (e.g. `sergw:ttyUSB0`)
- TXT records: `provider=sergw`

Disable mDNS by building without default features:

```
cargo build --no-default-features
```

### TUI overview

- Tabs: Overview (connections, throughput, events), Inspector (live dump)
- Inspector: formats (hex/ascii/dec), per‑device filter, pause/scroll
- Key hints in footer

### Reliability & behavior

- Serial auto‑reconnect on read/write failures; writer retries buffered write after reconnect
- TCP reader/writer per connection; on backpressure the connection is dropped rather than slowing others
- Raw byte forwarding (no framing, no higher protocols)

### Exit codes

- 2: no serial ports found for auto‑selection
- 3: multiple serial ports detected, explicit `--serial` required
- 4: bind‑like networking error (e.g. address in use)
- 5: serial open/error
- 1: other errors

### Development

- Build: `cargo build --all-features`
- Lint: `cargo clippy --all-targets --all-features -- -D warnings`
- Tests: unit tests + Linux PTY integration test

### License

GPL‑3.0‑or‑later

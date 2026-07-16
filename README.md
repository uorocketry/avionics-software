# uORocketry Avionics Software Monorepo

Firmware, shared avionics libraries, host applications, and ground-station deployment configuration live in this repository.

## Monorepo Structure

### Boards
Embedded software developed for each board lives in `boards/*`.

See [boards/](./boards/) for more details.

### Common Logic
Reusable drivers, peripherals, high-level logic, and utilities live in `common/*`.

See [common/](./common/).

### Host Applications

Independently runnable host applications live in [`apps/`](./apps/), including the Rust serial gateway, telemetry services, operator web interface, tile provider, and antenna tracker.

### Ground Station

The complete ground-station stack is defined in [`deployments/ground-station/`](./deployments/ground-station/). Development and field utilities live in [`tools/ground-station/`](./tools/ground-station/), with architecture notes in [`docs/ground-station/`](./docs/ground-station/).

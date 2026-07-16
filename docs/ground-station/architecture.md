# Architecture

Ground-station processes are separate where hardware ownership, safety, persistence, or resource boundaries require it. Docker Compose owns process supervision.

## Data Flow

```text
radio <-> sergw <-> telemetry-ingestor -> LibSQL <- ground-station-web <-> operator
                  ^                         ^
                  |                         |
                  +--- command-dispatcher --+

ground-station-web <-> tile-provider
```

## Applications

- **sergw** (`apps/sergw`, Rust) owns the physical serial device and exposes a raw TCP bridge. Its Compose service is optional when the radio is attached to another host.
- **telemetry-ingestor** (`apps/telemetry-ingestor`, Rust) decodes MAVLink and protobuf telemetry, writes LibSQL records, and reports service health.
- **command-dispatcher** (`apps/command-dispatcher`, Rust) consumes queued commands, sends them through `sergw`, updates delivery state, and emits bounded Ping keepalives.
- **ground-station-web** (`apps/ground-station-web`, SvelteKit/Bun) provides the operator UI and server-side APIs over LibSQL and local service controls.
- **tile-provider** (`apps/tile-provider`, Bun) owns map tile caching and offline download jobs.
- **antenna-tracker** (`apps/antenna-tracker`, Rust/Python) is an optional field application for GPS extraction, prediction, and antenna control.

LibSQL, schema migration, Traefik routing, and application wiring are defined in `deployments/ground-station/compose.yaml`. Diagnostic and setup programs under `tools/ground-station` are not deployed by default.

## Protocol Ownership

The ground station currently consumes `messages-prost` and the uORocketry MAVLink dialect from their existing Git repositories. Moving those generated protocol artifacts into `common/` is intentionally separate from this repository migration.

# Ground Station

The uORocketry ground station receives MAVLink telemetry, persists decoded protobuf messages, presents a compact operator interface, and dispatches commands back to the vehicle.

## Repository Locations

- Runtime applications: `apps/sergw`, `apps/telemetry-ingestor`, `apps/command-dispatcher`, `apps/ground-station-web`, and `apps/tile-provider`
- Optional antenna application: `apps/antenna-tracker`
- Deployment configuration and LibSQL schema: `deployments/ground-station`
- Development and field utilities: `tools/ground-station`

Start with [Architecture](architecture.md), then review the [requirements](requirements/index.md) and deployment README.

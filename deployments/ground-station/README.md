# Ground Station Deployment

This deployment composes the independently maintained applications under `apps/` with LibSQL and Traefik.

## Run

From the repository root:

```bash
docker compose -f deployments/ground-station/compose.yaml up --build
```

By default, telemetry services connect to `tcpout:sergw:5656`. To run the repository's Rust serial gateway with a local radio:

```bash
SERIAL_DEVICE=/dev/ttyUSB0 \
docker compose -f deployments/ground-station/compose.yaml --profile radio up --build
```

To use a gateway on another machine, leave the `radio` profile disabled and provide its MAVLink connection string:

```bash
GATEWAY_CONNECTION_STRING=tcpout:192.168.8.178:5656 \
docker compose -f deployments/ground-station/compose.yaml up --build
```

Runtime database and tile data are stored in ignored local state or Docker volumes. The `migration` service applies `db/seed.sql` when the stack starts.

Published ports can be overridden with `GROUND_STATION_HTTP_PORT`, `TRAEFIK_DASHBOARD_PORT`, `LIBSQL_PORT`, `TILES_PORT`, and `SERGW_PORT` when the defaults conflict with local services.

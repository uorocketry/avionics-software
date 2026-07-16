# Apps

Host-side applications that can be built, tested, and run independently:

- `sergw`: serial-to-TCP radio gateway
- `telemetry-ingestor`: MAVLink telemetry persistence
- `command-dispatcher`: database-backed outbound command delivery
- `ground-station-web`: operator interface and backend-for-frontend
- `tile-provider`: offline map tile storage and downloads
- `antenna-tracker`: optional GPS prediction and antenna control pipeline
- `argus`, `phil-cart`, `persistence`, and `grafana`: board and test infrastructure

Deployment composition belongs under `deployments/`; app directories should not contain copies of other applications.

# uORocketry Avionics Software Monorepo

Welcome to the uORocketry's avionics software monorepo. This is where we house all of our packages and crates from various boards and our ground station software. We're currently in the process of migrating our other repos to this monorepo so bear with us.

## Monorepo Structure
This monorepo is houses three main types of packages.

### Boards
Embedded software developed for each of our boards can be found at `boards/*`. 

See [boards/README.md](./boards/README.md) for more details.

### Common Logic
Common logic shared between boards and possibly ground station can be found at `common/*`. 

See [common/README.md](./common/README.md).

### Ground Station
Coming soon...
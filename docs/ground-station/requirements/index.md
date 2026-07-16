# Requirements

The monorepo pins Rust through `rust-toolchain.toml`. Ground-station development additionally requires:

- Docker with Compose for the integrated deployment
- Bun for the web, database migration, and tile-provider applications
- UV and Python 3.12+ for antenna and hardware utilities
- `pkg-config` and `libudev-dev` for host Rust applications that access serial devices

See [Development Environment](development-environment.md), [Docker](docker.md), [Bun](bun.md), and [Rust](rust.md).

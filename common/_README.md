
# TODO: This should contain common logic for things beyond the scope of boards, but currently only board logic is stored here. Because of this, the new README is more relevant  
# Common Software

This is where the common logic shared between boards and possibly ground station are stored.

## Setting Up Your Environment

### OS Packages
- Ubuntu: `sudo apt update && sudo apt install -y build-essential cmake protobuf-compiler libusb-1.0-0`
- macOS (Homebrew): `brew install cmake protobuf`
- Windows: use WSL2 (Ubuntu) or install the above equivalents; USB access works best via WSL2.

### Rust Toolchain
- Install Rustup: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Toolchain config is pinned in `rust-toolchain.toml`
- Verify: `rustc --version`, `rustup target list --installed`

### Workspace Utilities
- Install `cargo-make` (used by CI/tasks): `cargo install cargo-make`
- Install `rustfmt` used for linting: `rustup component add rustfmt`
- Install `cargo-sort` used for cleaning up `Cargo.toml` files. `cargo install cargo-sort`

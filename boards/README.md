# Avionics Boards Software

This is where we store our embedded software for each board we develop.

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

### Flashing and Logging
- Install probe-rs tools: `cargo install probe-rs-tools`
- On linux you need to setup udev rules (no sudo needed for USB): 
  -`curl -L https://probe.rs/files/udev/60-probe-rs.rules | sudo tee /etc/udev/rules.d/60-probe-rs.rules`. This link might not be maintained always. Might need to google how to setup the usb device rules with `probe-rs`.
  - `sudo udevadm control --reload-rules && sudo udevadm trigger`
- With a JLINK probe connected to your USB, verify that is visible: `probe-rs list`

## Running code 
Cargo runner is already configured to use probe-rs with the correct chip. You can simply run `cargo run --bin {board}` and it will build and flash to the chip.

## Tests 
- To run tests on the embedded device `cargo make test-embedded`
- To run tests that can be run on your host machine `cargo make test-host`

## Helpful VSCode Extensions 
- `probe-rs.probe-rs-debugger`
- `rust-lang.rust-analyzer`

## Useful Tools
### Cargo Size
Shows you the memory size breakdown of a compiled Rust binary: `rustup component add llvm-tools-preview`

To run against your build `cargo size --bin {board}`
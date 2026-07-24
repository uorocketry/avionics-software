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

### Cargo Bloat
To see if use of a generic causing monomorphization bloat. We can use cargo bloat. `cargo install cargo-bloat cargo-binutils rustfilt`
`cargo bloat --release -n 200 --demangle`

## Common Issues

1) **Rust-analyzer is complaining about my target triple being my host PC's target instead of embedded when looking at embedded code**.
```
embassy_executor::main: proc macro server error: Cannot create expander for ./target/debug/deps/libembassy_executor_macros-dfe0a64b5f1613f4.dylib:
mismatched ABI expected: `rustc 1.89.0 (29483883e 2025-08-04)`, got `rustc 1.91.0-nightly (7aef4bec4 2025-09-01)
```
This is common because rust-analyzer picks up what target to use based on the vscode's working directory's `.cargo/Config.toml` build target. If you install the "Rust Target" extension you can dynamically jump around targets. Or alternatively open the folder for the board as your vscode's working directory instead of the entire monorepo and rust-analyzer will pick up the `.cargo/Config.toml` of the board's folder. We're using the `thumbv7em-none-eabihf` target triple.
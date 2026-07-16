# Development Environment

Linux is the primary development and deployment environment. Windows contributors should use WSL.

```bash
git clone https://github.com/uorocketry/avionics-software
cd avionics-software
sudo apt update
sudo apt install -y git pkg-config libudev-dev
```

Install Docker, Bun, and UV using the linked requirement pages or their official installers. Open the repository root in your editor so Rust Analyzer can discover the shared Cargo workspace.

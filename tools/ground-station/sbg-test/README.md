## SBG Test Utilities

Tools and scripts to work with SBG Systems devices. This folder provides:
- Two native tools from the SBG ECom library (`sbgBasicLogger`, `sbgEComApi`)
- Small Python helpers for reading and viewing data

### Requirements
- CMake and a C/C++ toolchain
- Git
- Python 3.10+ (optional, for the helper scripts)

### Serial/USB access (run without root)
Most systems restrict access to `/dev/ttyUSB*` or `/dev/ttyACM*` devices to specific groups.
If you can’t open the device, run the helper which will suggest and add the right groups:

```bash
cd utils/sbg-test
python3 fix_perms.py
# Log out/in (or reboot) for group changes to apply.
```

### Build the SBG tools
This compiles the SBG ECom tools and places the binaries in this folder.

```bash
cd utils/sbg-test
python3 build_sbg_utils.py
```
- The script clones/updates `sbgECom`, configures CMake with tools enabled, builds, and copies `sbgBasicLogger` and `sbgEComApi` here.
- Works with CMake’s native generator on your platform (Make/Ninja/Visual Studio/MSBuild, etc.).

### Quick usage
- Log to console from a serial device:
  ```bash
./sbgBasicLogger -s /dev/ttyUSB0 -r 115200 -p
  ```
- Inspect the REST-style API client:
  ```bash
./sbgEComApi --help
  ```

### Python helpers
- `sbg_reader.py`: Typed, high-level reader around `sbgBasicLogger` output. Provides structured callbacks/messages for EKF, GNSS, IMU, air data, etc.
- `main.py`: Simple terminal UI (Textual) to stream, tab, and view SBG data.
- `latest_measurement.py`: One-shot, labeled readout of the latest values across key data types.
- `gps_orientation.py`: Minimal GPS + orientation viewer using the reader.
- `fix_perms.py`: Adds your user to groups that own serial devices so you don’t need root.

That’s it—ensure serial permissions, build the tools, and you’re ready to log or script against your SBG device.

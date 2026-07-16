# Remote Test Utility

A simple joystick/gamepad input testing tool that reads and displays events from `/dev/input/js0`.

## What It Does

- Monitors joystick input events in real-time
- Displays button presses/releases and axis movements
- Automatically reconnects if the device is disconnected
- Normalizes axis values to -1.0 to 1.0 range

## Usage

```bash
python3 remote.py
```

## Output Example

```
[14:30:15] Listening on /dev/input/js0 (Ctrl-C to exit)…
[1234567 ms] Button A DOWN
[1234568 ms] Button A UP
[1234569 ms] Axis LX → 0.500
[1234570 ms] Axis LY → -0.250
```

## Requirements

- Linux system with joystick/gamepad connected
- Python 3
- Access to `/dev/input/js0`

## Supported Inputs

**Buttons**: A, B, X, Y, LB, RB, Back, Start, Guide, LThumb, RThumb
**Axes**: LX, LY, LT, RX, RY, RT

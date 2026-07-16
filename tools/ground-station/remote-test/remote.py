#!/usr/bin/env python3
import errno
import fcntl
import os
import struct
import threading
import time
from typing import Dict, Optional


EVENT_FMT = 'IhBB'
EVENT_SIZE = struct.calcsize(EVENT_FMT)

JS_EVENT_BUTTON = 0x01
JS_EVENT_AXIS = 0x02
JS_EVENT_INIT = 0x80


BUTTON_INDEX_TO_NAME: Dict[int, str] = {
    0: 'A', 1: 'B', 2: 'X', 3: 'Y',
    4: 'LB', 5: 'RB', 6: 'Back', 7: 'Start',
    8: 'Guide', 9: 'LThumb', 10: 'RThumb',
}

AXIS_INDEX_TO_NAME: Dict[int, str] = {
    0: 'LX', 1: 'LY', 2: 'LT',
    3: 'RX', 4: 'RY', 5: 'RT',
}

BUTTON_NAME_TO_INDEX: Dict[str, int] = {v: k for k, v in BUTTON_INDEX_TO_NAME.items()}
AXIS_NAME_TO_INDEX: Dict[str, int] = {v: k for k, v in AXIS_INDEX_TO_NAME.items()}


def _normalize_axis(value: int) -> float:
    if value <= -32767:
        return -1.0
    if value >= 32767:
        return 1.0
    return value / 32767.0


class XboxController:
    """Simple Linux joystick reader for Xbox controllers using /dev/input/js*.

    Exposes a minimal API expected by manual.py:
      - start() -> bool
      - stop() -> None
      - is_connected() -> bool
      - get_button(name: str) -> bool
      - get_axis(name: str) -> float in [-1.0, 1.0]
    """

    def __init__(self, deadzone: float = 0.1, device_path: str = '/dev/input/js0') -> None:
        self.device_path: str = device_path
        self.deadzone: float = max(0.0, min(0.9, deadzone))
        self._thread: Optional[threading.Thread] = None
        self._stop_event = threading.Event()
        self._connected: bool = False
        self._fd: Optional[int] = None
        self._lock = threading.Lock()
        self._buttons: Dict[str, bool] = {name: False for name in BUTTON_NAME_TO_INDEX.keys()}
        self._axes: Dict[str, float] = {name: 0.0 for name in AXIS_NAME_TO_INDEX.keys()}

    def start(self) -> bool:
        if self._thread and self._thread.is_alive():
            return True
        self._stop_event.clear()
        self._thread = threading.Thread(target=self._reader_loop, name='XboxControllerReader', daemon=True)
        self._thread.start()
        # Give the thread a short time to attempt initial open
        time.sleep(0.1)
        return True

    def stop(self) -> None:
        self._stop_event.set()
        if self._thread and self._thread.is_alive():
            self._thread.join(timeout=1.5)
        self._close_device()

    def is_connected(self) -> bool:
        return self._connected

    def get_button(self, name: str) -> bool:
        with self._lock:
            return bool(self._buttons.get(name, False))

    def get_axis(self, name: str) -> float:
        with self._lock:
            value = float(self._axes.get(name, 0.0))
        if abs(value) < self.deadzone:
            return 0.0
        return value

    # Internal
    def _open_device(self) -> None:
        try:
            fd = os.open(self.device_path, os.O_RDONLY | os.O_NONBLOCK)
            # Ensure non-blocking
            fl = fcntl.fcntl(fd, fcntl.F_GETFL)
            fcntl.fcntl(fd, fcntl.F_SETFL, fl | os.O_NONBLOCK)
            self._fd = fd
            self._connected = True
        except FileNotFoundError:
            self._connected = False
        except PermissionError:
            self._connected = False

    def _close_device(self) -> None:
        if self._fd is not None:
            try:
                os.close(self._fd)
            except OSError:
                pass
            finally:
                self._fd = None
        self._connected = False

    def _reader_loop(self) -> None:
        last_attempt = 0.0
        while not self._stop_event.is_set():
            # Ensure device is open
            if self._fd is None:
                now = time.time()
                if now - last_attempt < 0.5:
                    time.sleep(0.05)
                    continue
                last_attempt = now
                self._open_device()
                if self._fd is None:
                    time.sleep(0.3)
                    continue

            # Read any available events
            try:
                data = os.read(self._fd, EVENT_SIZE)
                if not data:
                    time.sleep(0.01)
                    continue
                if len(data) < EVENT_SIZE:
                    continue
                timestamp, value, ev_type, number = struct.unpack(EVENT_FMT, data)
                if ev_type & JS_EVENT_INIT:
                    continue
                if ev_type & JS_EVENT_BUTTON:
                    name = BUTTON_INDEX_TO_NAME.get(number)
                    if name is not None:
                        with self._lock:
                            self._buttons[name] = (value != 0)
                elif ev_type & JS_EVENT_AXIS:
                    name = AXIS_INDEX_TO_NAME.get(number)
                    if name is not None:
                        norm = _normalize_axis(value)
                        with self._lock:
                            self._axes[name] = norm
                # Gentle yield
                time.sleep(0.001)
            except BlockingIOError:
                time.sleep(0.01)
                continue
            except OSError as e:
                if e.errno == errno.ENODEV:
                    # Device disconnected
                    self._close_device()
                    time.sleep(0.3)
                    continue
                # Other I/O error: reset and retry
                self._close_device()
                time.sleep(0.2)
                continue


# This module is intended to be imported by manual.py; no CLI entry point.

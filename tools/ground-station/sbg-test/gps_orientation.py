#!/usr/bin/env python3
"""
A Textual application to display live GPS and Orientation data from an SBG device.
"""

import os
import glob
import sys
import threading
import math
from typing import Optional

from rich.text import Text
from textual.app import App, ComposeResult
from textual.widgets import Header, Footer, Static
from textual.reactive import reactive
from sbg_reader import SBGReader, SbgEkfNavData, SbgEkfEulerData, SbgEkfQuatData


def find_ftdi_device(vendor_id="0403", product_id="6001") -> Optional[str]:
    """Find FTDI device by vendor and product ID."""
    import glob

    usb_devs = glob.glob("/dev/ttyUSB*")
    for dev in usb_devs:
        try:
            tty_name = os.path.basename(dev)
            sys_tty_path = f"/sys/class/tty/{tty_name}/device"
            sys_usb_path = os.path.realpath(sys_tty_path)
            for _ in range(3):
                id_vendor = os.path.join(sys_usb_path, "idVendor")
                id_product = os.path.join(sys_usb_path, "idProduct")
                if os.path.exists(id_vendor) and os.path.exists(id_product):
                    with open(id_vendor) as f:
                        vid = f.read().strip().lower()
                    with open(id_product) as f:
                        pid = f.read().strip().lower()
                    if vid == vendor_id.lower() and pid == product_id.lower():
                        return dev
                sys_usb_path = os.path.dirname(sys_usb_path)
        except Exception:
            continue
    return None


def quaternion_to_euler(qw: float, qx: float, qy: float, qz: float) -> tuple[float, float, float]:
    """
    Convert quaternion to euler angles (roll, pitch, yaw) in degrees.

    Args:
        qw, qx, qy, qz: Quaternion components

    Returns:
        tuple: (roll, pitch, yaw) in degrees
    """
    # Normalize quaternion
    norm = math.sqrt(qw*qw + qx*qx + qy*qy + qz*qz)
    if norm == 0:
        return 0.0, 0.0, 0.0

    qw /= norm
    qx /= norm
    qy /= norm
    qz /= norm

    # Convert to euler angles (roll, pitch, yaw)
    # Roll (x-axis rotation)
    sinr_cosp = 2 * (qw * qx + qy * qz)
    cosr_cosp = 1 - 2 * (qx * qx + qy * qy)
    roll = math.atan2(sinr_cosp, cosr_cosp)

    # Pitch (y-axis rotation)
    sinp = 2 * (qw * qy - qz * qx)
    if abs(sinp) >= 1:
        pitch = math.copysign(math.pi / 2, sinp)  # use 90 degrees if out of range
    else:
        pitch = math.asin(sinp)

    # Yaw (z-axis rotation)
    siny_cosp = 2 * (qw * qz + qx * qy)
    cosy_cosp = 1 - 2 * (qy * qy + qz * qz)
    yaw = math.atan2(siny_cosp, cosy_cosp)

    # Convert to degrees
    roll_deg = math.degrees(roll)
    pitch_deg = math.degrees(pitch)
    yaw_deg = math.degrees(yaw)

    return roll_deg, pitch_deg, yaw_deg


class GPSDisplay(Static):
    """A widget to display GPS data."""
    lat = reactive(0.0)
    lon = reactive(0.0)

    def update_data(self, data: SbgEkfNavData):
        self.lat = data.latitude
        self.lon = data.longitude

    def render(self) -> Text:
        return Text.from_markup(f"[b]Latitude:[/b]  {self.lat:9.6f}°\n[b]Longitude:[/b] {self.lon:9.6f}°")


class OrientationDisplay(Static):
    """A widget to display orientation data."""
    roll = reactive(0.0)
    pitch = reactive(0.0)
    yaw = reactive(0.0)

    def update_data(self, data: SbgEkfQuatData):
        # Convert quaternion to euler angles
        self.roll, self.pitch, self.yaw = quaternion_to_euler(data.qW, data.qX, data.qY, data.qZ)

    def render(self) -> Text:
        return Text.from_markup(
            f"[b]Roll:[/b]  {self.roll:8.3f}°\n[b]Pitch:[/b] {self.pitch:8.3f}°\n[b]Yaw:[/b]   {self.yaw:8.3f}°"
        )


class HeadingDisplay(Static):
    """A widget to display heading data."""
    ekf_heading = reactive(0.0)
    gnss_heading = reactive(0.0)
    gnss_heading_available = reactive(False)

    def update_ekf_data(self, data: SbgEkfQuatData):
        # Convert quaternion to euler and get yaw as heading
        _, _, yaw = quaternion_to_euler(data.qW, data.qX, data.qY, data.qZ)
        self.ekf_heading = yaw

    def update_gnss_data(self, data):
        # This method is kept for compatibility but GNSS heading is not available
        pass

    def render(self) -> Text:
        if self.gnss_heading_available:
            return Text.from_markup(
                f"[b]EKF Heading:[/b]  {self.ekf_heading:8.3f}°\n[b]GNSS Heading:[/b] {self.gnss_heading:8.3f}°"
            )
        else:
            return Text.from_markup(
                f"[b]EKF Heading:[/b]  {self.ekf_heading:8.3f}°\n[b]GNSS Heading:[/b] [dim]Not available[/dim]"
            )


class GPSOrientationApp(App):
    """A Textual app to display SBG GPS and orientation data."""

    TITLE = "SBG GPS + Orientation + Heading"
    CSS_PATH = None
    BINDINGS = [("q", "quit", "Quit")]

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self._sbg_reader = SBGReader()
        self._reader_thread: Optional[threading.Thread] = None

    def compose(self) -> ComposeResult:
        yield Header()
        yield Static("Initializing...", id="status")
        yield GPSDisplay(id="gps")
        yield OrientationDisplay(id="orientation")
        yield HeadingDisplay(id="heading")
        yield Footer()

    def on_mount(self) -> None:
        """Called when the app is mounted."""
        # Start the reader thread after the app is fully mounted
        self.call_after_refresh(self._start_reader_thread)

    def _start_reader_thread(self) -> None:
        """Start the reader thread after the UI is fully ready."""
        self._reader_thread = threading.Thread(target=self._run_reader, daemon=True)
        self._reader_thread.start()

    def _run_reader(self) -> None:
        """The target for the reader thread, responsible for finding and starting the device reader."""
        status_widget = self.query_one("#status", Static)
        self.call_from_thread(status_widget.update, "Searching for FTDI device...")
        device = find_ftdi_device()

        if not device:
            status_widget = self.query_one("#status", Static)
            self.call_from_thread(status_widget.update, Text("Error: FTDI device not found.", style="bold red"))
            return

        status_widget = self.query_one("#status", Static)
        self.call_from_thread(status_widget.update, f"Device found at [bold green]{device}[/]. Starting reader...")

        try:
            self._sbg_reader.subscribe('nav', self.on_nav_data)
            self._sbg_reader.subscribe('quat', self.on_quat_data)
            self._sbg_reader.start(device_path=device)

            status_widget = self.query_one("#status", Static)
            self.call_from_thread(status_widget.update, "Reader started. Waiting for data...")
        except (FileNotFoundError, RuntimeError) as e:
            status_widget = self.query_one("#status", Static)
            self.call_from_thread(status_widget.update, Text(f"Error: {e}", style="bold red"))

    def on_unmount(self) -> None:
        """Called when the app is unmounted."""
        self._sbg_reader.stop()

    def on_nav_data(self, data: SbgEkfNavData) -> None:
        """Callback for new navigation data."""
        gps_widget = self.query_one("#gps")
        self.call_from_thread(gps_widget.update_data, data)

    def on_quat_data(self, data: SbgEkfQuatData) -> None:
        """Callback for new quaternion data."""
        orientation_widget = self.query_one("#orientation")
        heading_widget = self.query_one("#heading")
        self.call_from_thread(orientation_widget.update_data, data)
        self.call_from_thread(heading_widget.update_ekf_data, data)

    def action_quit(self) -> None:
        """An action to quit the app."""
        self.exit()


if __name__ == "__main__":
    app = GPSOrientationApp()
    app.run()

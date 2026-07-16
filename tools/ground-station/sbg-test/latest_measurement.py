#!/usr/bin/env python3
"""
Standalone script to get the latest measurement from SBG device with labels.

This script connects to the SBG device using the sbgBasicLogger binary and
displays the latest measurement for each data type with proper labels.
"""

import os
import subprocess
import time
import threading
import signal
import sys
from typing import List


def find_ftdi_device(vendor_id="0403", product_id="6001"):
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


class SBGMeasurementReader:
    """Class to read and parse SBG measurements.

    The data types and field order are based on the sbgBasicLogger source code:
    - Main registration order: utils/sbg-test/sbgECom/tools/sbgBasicLogger/src/loggerApp.cpp (lines 50-90)
    - GNSS field definitions: utils/sbg-test/sbgECom/tools/sbgBasicLogger/src/loggerEntry/loggerEntryGnss.cpp (lines 100-115)
    - EKF field definitions: utils/sbg-test/sbgECom/tools/sbgBasicLogger/src/loggerEntry/loggerEntryEkf.cpp
    - IMU field definitions: utils/sbg-test/sbgECom/tools/sbgBasicLogger/src/loggerEntry/loggerEntryImu.cpp
    """

    # Define the labels for each data type based on sbgBasicLogger source code
    DATA_LABELS = {
        'euler': [
            'Status', 'Roll (deg)', 'Pitch (deg)', 'Yaw (deg)',
            'Roll Std (deg)', 'Pitch Std (deg)', 'Yaw Std (deg)',
            'Mag Heading (deg)', 'Mag Decl (deg)', 'Mag Incl (deg)'
        ],
        'quat': [
            'Status', 'qW', 'qX', 'qY', 'qZ',
            'Roll Std (deg)', 'Pitch Std (deg)', 'Yaw Std (deg)',
            'Mag Decl (deg)', 'Mag Incl (deg)'
        ],
        'nav': [
            'Status', 'Vel N (m/s)', 'Vel E (m/s)', 'Vel D (m/s)',
            'Vel Std N (m/s)', 'Vel Std E (m/s)', 'Vel Std D (m/s)',
            'Latitude (deg)', 'Longitude (deg)', 'Altitude (m)',
            'Lat Std (m)', 'Lon Std (m)', 'Alt Std (m)', 'Undulation (m)'
        ],
        'airData': [
            'Status', 'Pressure (hPa)', 'Altitude (m)',
            'Temperature (C)', 'Air Speed (m/s)', 'Air Speed Std (m/s)'
        ],
        'imuData': [
            'Status', 'Accel X (m/s²)', 'Accel Y (m/s²)', 'Accel Z (m/s²)',
            'Gyro X (deg/s)', 'Gyro Y (deg/s)', 'Gyro Z (deg/s)', 'Temperature (C)'
        ],
        'gnss1Vel': [
            'Status', 'GPS Time (ms)', 'Vel N (m/s)', 'Vel E (m/s)', 'Vel D (m/s)',
            'Vel Std N (m/s)', 'Vel Std E (m/s)', 'Vel Std D (m/s)',
            'Course (deg)', 'Course Std (deg)'
        ],
        'gnss1Pos': [
            'Status', 'Status Ext', 'GPS Time (ms)', 'Latitude (deg)', 'Longitude (deg)', 'Altitude (m)',
            'Undulation (m)', 'Lat Std (m)', 'Lon Std (m)', 'Alt Std (m)',
            'Satellites Tracked', 'Satellites Used', 'Base Station ID', 'Differential Age'
        ],
        'status': [
            'Status', 'Main Power', 'IMU Power', 'GPS Power', 'Settings'
        ],
        'utcTime': [
            'Status', 'Year', 'Month', 'Day', 'Hour', 'Minute', 'Second',
            'Nanosecond', 'GpsTimeOfWeek (s)', 'Flags', 'Valid', 'Time (s)'
        ],
        'gnss1Hdt': [
            'Status', 'GPS Time (ms)', 'Heading (deg)', 'Heading Std (deg)',
            'Pitch (deg)', 'Pitch Std (deg)', 'Baseline (m)', 'Satellites Tracked', 'Satellites Used'
        ],
        'gnss1Sat': [
            'Satellites Count'
        ],
        'imuShort': [
            'Status', 'Delta Vel X (m/s)', 'Delta Vel Y (m/s)', 'Delta Vel Z (m/s)',
            'Delta Angle X (deg)', 'Delta Angle Y (deg)', 'Delta Angle Z (deg)', 'Temperature (C)'
        ],
        'imuFast': [
            'Status', 'Delta Vel X (m/s)', 'Delta Vel Y (m/s)', 'Delta Vel Z (m/s)',
            'Delta Angle X (deg)', 'Delta Angle Y (deg)', 'Delta Angle Z (deg)', 'Temperature (C)'
        ],
        'mag': [
            'Status', 'Mag X (uT)', 'Mag Y (uT)', 'Mag Z (uT)', 'Temperature (C)'
        ],
        'diag': [
            'Status', 'Main Loop Time (us)', 'Computation Load (%)', 'CPU Temperature (C)'
        ],
        'velBody': [
            'Status', 'Vel X (m/s)', 'Vel Y (m/s)', 'Vel Z (m/s)',
            'Vel Std X (m/s)', 'Vel Std Y (m/s)', 'Vel Std Z (m/s)'
        ]
    }

    def __init__(self):
        self.latest_measurements = {}
        self.running = False
        self.process = None

    def format_number(self, value_str: str) -> str:
        """Format a number string - round floats to 3 decimals, keep integers as-is."""
        try:
            # First try to parse as int
            int_val = int(value_str)
            return str(int_val)
        except ValueError:
            try:
                # If that fails, try as float
                float_val = float(value_str)
                return f"{float_val:.3f}"
            except ValueError:
                # If both fail, return original string
                return value_str

    def parse_line(self, line: str) -> None:
        """Parse a line from sbgBasicLogger output."""
        if not line or line.startswith("*"):
            return

        parts = line.strip().split()
        if not parts:
            return

        message_type = parts[0].rstrip(":")
        data = parts[1:]

        # Format the data nicely
        formatted_data = []
        for item in data:
            formatted_data.append(self.format_number(item))

        # Store the latest measurement
        self.latest_measurements[message_type] = formatted_data

    def read_from_device(self) -> None:
        """Read data from the SBG device."""
        try:
            print("Looking for FTDI device...")
            dev_path = find_ftdi_device("0403", "6001")
            if not dev_path:
                print("Error: FTDI USB device not found")
                return

            print(f"Found FTDI device at: {dev_path}")

            if not os.path.exists("./sbgBasicLogger"):
                print("Error: sbgBasicLogger executable not found")
                return

            print("Starting sbgBasicLogger...")
            cmd = ["./sbgBasicLogger", "-s", dev_path, "-r", "115200", "-p"]
            print(f"Command: {' '.join(cmd)}")

            self.process = subprocess.Popen(
                cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                text=True, bufsize=1
            )

            time.sleep(1)
            if self.process.poll() is not None:
                err = self.process.stderr.read().strip()
                print(f"sbgBasicLogger failed with: {err}")
                return

            print("sbgBasicLogger started successfully, reading data...")
            print("Press Ctrl+C to stop and display latest measurements\n")

            if self.process.stdout:
                for line in iter(self.process.stdout.readline, ""):
                    if not self.running:
                        break
                    if line.strip():
                        self.parse_line(line)

        except FileNotFoundError as e:
            print(f"FileNotFoundError: {e}")
        except Exception as e:
            print(f"Unexpected error: {e}")

    def print_message(self, msg_type: str, data: List[str]) -> None:
        """Prints a single formatted message."""
        labels = self.DATA_LABELS.get(msg_type, [])

        print(f"\n{msg_type.upper()} DATA:")
        print("-" * 40)

        # Display data with labels
        for i, (label, value) in enumerate(zip(labels, data)):
            if i < len(data):
                print(f"  {label:<20}: {value}")
            else:
                print(f"  {label:<20}: N/A")

        # If we have more data than labels, show the extra data
        for i in range(len(labels), len(data)):
            print(f"  Data[{i}]: {data[i]}")

    def display_latest_measurements(self) -> None:
        """Display the latest measurements with labels and a summary."""
        if not self.latest_measurements:
            print("No measurements received yet.")
            return

        print("\n" + "="*80)
        print("LATEST SBG MEASUREMENTS")
        print("="*80)

        # Display important measurements first, then any others.
        message_order = ['status', 'diag', 'imuData', 'euler', 'nav', 'airData', 'gnss1Vel', 'gnss1Pos']

        displayed_messages = set()

        for msg_type in message_order:
            if msg_type in self.latest_measurements:
                self.print_message(msg_type, self.latest_measurements[msg_type])
                displayed_messages.add(msg_type)

        for msg_type, data in self.latest_measurements.items():
            if msg_type not in displayed_messages:
                self.print_message(msg_type, data)
                displayed_messages.add(msg_type)

        print("\n" + "="*80)
        print("SUMMARY")
        print("="*80)

        # Check for unknown message types that were received
        unknown_messages = set(self.latest_measurements.keys()) - set(self.DATA_LABELS.keys())
        if unknown_messages:
            print("\nUnknown message types received (no labels defined):")
            for msg_type in unknown_messages:
                print(f"  - {msg_type}")
        else:
            print("\nAll received message types are known.")

        # Check for defined labels for which no data was received
        missing_messages = set(self.DATA_LABELS.keys()) - set(self.latest_measurements.keys())
        if missing_messages:
            print("\nNo data received for the following message types:")
            for msg_type in sorted(list(missing_messages)):
                print(f"  - {msg_type}")
        else:
            print("\nData received for all known message types.")

    def start(self) -> None:
        """Start reading from the device."""
        self.running = True

        # Start reading thread
        reader_thread = threading.Thread(target=self.read_from_device, daemon=True)
        reader_thread.start()

        try:
            # Keep the main thread alive
            while self.running:
                time.sleep(0.1)
        except KeyboardInterrupt:
            print("\nStopping...")
            self.stop()

    def stop(self) -> None:
        """Stop reading from the device."""
        self.running = False
        if self.process:
            self.process.terminate()
            try:
                self.process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                self.process.kill()

        # Display the latest measurements
        self.display_latest_measurements()


def main():
    """Main function."""
    print("SBG Latest Measurement Reader")
    print("=" * 40)

    reader = SBGMeasurementReader()

    # Set up signal handler for graceful shutdown
    def signal_handler(signum, frame):
        print("\nReceived signal to stop...")
        reader.stop()
        sys.exit(0)

    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)

    try:
        reader.start()
    except Exception as e:
        print(f"Error: {e}")
        reader.stop()


if __name__ == "__main__":
    main()

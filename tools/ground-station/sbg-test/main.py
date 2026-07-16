import os
import subprocess
import time
import threading
from collections import deque
from typing import Dict, Deque, List, Optional

from textual.app import App, ComposeResult
from textual.containers import Horizontal, Vertical
from textual.css.query import NoMatches
from textual.widgets import Header, Footer, TabbedContent, TabPane, RichLog, Label
from textual.reactive import reactive
from textual import work, log
from textual.message import Message


def debug_log(message: str):
    """Write debug messages to log file"""
    with open("./log.txt", "a") as f:
        timestamp = time.strftime("%H:%M:%S")
        f.write(f"[{timestamp}] {message}\n")
        f.flush()


def find_ftdi_device(vendor_id="0403", product_id="6001") -> Optional[str]:
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


class UpdateMessage(Message):
    def __init__(self, message_type: str, data: List[str], timestamp: str, log_text: str):
        super().__init__()
        self.message_type = message_type
        self.data = data
        self.timestamp = timestamp
        self.log_text = log_text


class EllipseReaderApp(App):
    CSS = """
    Vertical#main-container {
        height: 100%;
    }

    Label#status {
        height: 1;
        width: 100%;
        content-align: left middle;
    }

    Label#utc {
        height: 1;
        width: 100%;
        content-align: left middle;
    }

    TabbedContent#message-tabs {
        height: 1fr;
        margin-top: 1;
    }
    """

    BINDINGS = [
        ("d", "clear_active_data", "Clear Data"),
        ("q", "quit", "Quit"),
    ]

    connection_status = reactive("Awaiting Data...")
    last_utc_time = reactive("N/A")

    def __init__(self):
        super().__init__()
        # Clean up log file at startup
        if os.path.exists("./log.txt"):
            os.remove("./log.txt")
        debug_log("=== EllipseReader Starting ===")

        self.running = threading.Event()
        self.max_messages = 10
        self.message_queues: Dict[str, Deque[str]] = {}
        self.reader_thread = None

    def compose(self) -> ComposeResult:
        with Vertical(id="main-container"):
            yield Label(f"Status: {self.connection_status}", id="status")
            yield Label(f"UTC:    {self.last_utc_time}", id="utc")
            yield TabbedContent(id="message-tabs")

    def action_clear_active_data(self) -> None:
        try:
            tabs = self.query_one("#message-tabs", TabbedContent)
            if tabs.active:
                pane = tabs.get_pane(tabs.active)
                if pane:
                    rich_log = pane.query_one(RichLog)
                    rich_log.clear()
        except (NoMatches, ValueError):
            pass

    def watch_connection_status(self, value: str) -> None:
        """Update the status label whenever the reactive changes."""
        try:
            self.query_one("#status", Label).update(f"Status: {value}")
        except NoMatches:
            # ignore until the label actually exists
            pass

    def watch_last_utc_time(self, value: str) -> None:
        """Update the UTC label whenever the reactive changes."""
        try:
            self.query_one("#utc", Label).update(f"UTC:    {value}")
        except NoMatches:
            pass

    def on_update_message(self, message: UpdateMessage) -> None:
        """Handle UI updates, capping per-type logs and auto-creating tabs."""
        debug_log(f"Received UpdateMessage for type: {message.message_type}")

        queue = self.message_queues.setdefault(
            message.message_type, deque(maxlen=self.max_messages)
        )
        queue.append(message.log_text)

        # set reactive to 'Connected' once we get any data
        if self.connection_status != "Connected":
            self.connection_status = "Connected"

        try:
            tabs = self.query_one("#message-tabs", TabbedContent)

            # Check if tab already exists by looking at all tab panes
            existing_pane = None
            for pane in tabs.query(TabPane):
                if pane.id == message.message_type:
                    existing_pane = pane
                    debug_log(f"Tab '{message.message_type}' already exists")
                    break

            if not existing_pane:
                # Tab doesn't exist, create it
                debug_log(f"Creating new tab: {message.message_type}")
                pane = TabPane(message.message_type, id=message.message_type)
                rich_log = RichLog(wrap=True, highlight=False, auto_scroll=True)
                pane.compose_add_child(rich_log)
                tabs.add_pane(pane)

                # Set as active if it's the first tab
                if not tabs.active:
                    tabs.active = message.message_type
                    debug_log(f"Set {message.message_type} as active tab")
                existing_pane = pane
                debug_log(f"Successfully created tab: {message.message_type}")

            # refresh just this tab
            if existing_pane:
                try:
                    log_widget = existing_pane.query_one(RichLog)
                    log_widget.clear()
                    for entry in queue:
                        log_widget.write(entry)
                    debug_log(f"Updated tab '{message.message_type}' with {len(queue)} entries")
                except NoMatches as e:
                    debug_log(f"Could not find RichLog in pane {message.message_type}: {e}")

        except NoMatches as e:
            debug_log(f"NoMatches error in on_update_message: {e}")
        except Exception as e:
            debug_log(f"Error in on_update_message: {e}")
            import traceback
            debug_log(f"Traceback: {traceback.format_exc()}")

    def read_from_device_thread(self) -> None:
        """Thread function to read from device"""
        debug_log("Starting device reader thread")
        try:
            debug_log("Looking for FTDI device...")
            dev_path = find_ftdi_device("0403", "6001")
            if not dev_path:
                debug_log("FTDI device not found")
                self.connection_status = "Error: FTDI USB device not found"
                return

            debug_log(f"Found FTDI device at: {dev_path}")

            if not os.path.exists("./sbgBasicLogger"):
                debug_log("sbgBasicLogger executable not found")
                self.connection_status = "Error: sbgBasicLogger missing"
                return

            debug_log("Starting sbgBasicLogger...")
            cmd = ["./sbgBasicLogger", "-s", dev_path, "-r", "115200", "-p"]
            debug_log(f"Command: {' '.join(cmd)}")

            proc = subprocess.Popen(
                cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                text=True, bufsize=1
            )
            time.sleep(1)
            if proc.poll() is not None:
                err = proc.stderr.read().strip()
                debug_log(f"sbgBasicLogger failed with: {err}")
                self.connection_status = f"sbgBasicLogger failed: {err}"
                return

            debug_log("sbgBasicLogger started successfully, reading lines...")
            self.connection_status = "Connected"

            line_count = 0
            if proc.stdout:
                for line in iter(proc.stdout.readline, ""):
                    if not self.running.is_set():
                        debug_log("Thread stopped by running flag")
                        break
                    if line.strip():
                        line_count += 1
                        if line_count <= 5:  # Log first 5 lines for debugging
                            debug_log(f"Raw line {line_count}: {repr(line.strip())}")
                        self.parse_line(line)

        except FileNotFoundError as e:
            debug_log(f"FileNotFoundError: {e}")
            self.connection_status = "Error: sbgBasicLogger not found"
        except Exception as e:
            debug_log(f"Unexpected error: {e}")
            self.connection_status = f"Error: {e}"

    def format_number(self, value_str: str) -> str:
        """Format a number string - round floats to 2 decimals, keep integers as-is"""
        try:
            # First try to parse as int
            int_val = int(value_str)
            return str(int_val)
        except ValueError:
            try:
                # If that fails, try as float
                float_val = float(value_str)
                return f"{float_val:.2f}"
            except ValueError:
                # If both fail, return original string
                return value_str

    def parse_line(self, line: str) -> None:
        if not line or line.startswith("*"):
            return
        parts = line.strip().split()
        if not parts:
            return

        message_type = parts[0].rstrip(":")
        data = parts[1:]

        # Format the data nicely - remove the header and format numbers
        formatted_data = []
        for item in data:
            formatted_data.append(self.format_number(item))

        # Create a clean log text without the header
        clean_log_text = "  ".join(formatted_data)

        debug_log(f"Parsed - Type: '{message_type}', Data count: {len(data)}")

        # Use current time as timestamp
        timestamp = time.strftime("%H:%M:%S")

        # Post the message to create tabs with clean data
        debug_log(f"Posting UpdateMessage for type: {message_type}")
        self.call_from_thread(self.post_message, UpdateMessage(message_type, data, timestamp, clean_log_text))

    def on_mount(self) -> None:
        debug_log("App mounted, starting reader thread")
        self.running.set()
        # Start the background thread to read from device
        self.reader_thread = threading.Thread(target=self.read_from_device_thread, daemon=True)
        self.reader_thread.start()

    def on_unmount(self) -> None:
        debug_log("App unmounting, stopping threads")
        self.running.clear()
        if self.reader_thread and self.reader_thread.is_alive():
            self.reader_thread.join(timeout=1.0)


if __name__ == "__main__":
    app = EllipseReaderApp()
    app.run()
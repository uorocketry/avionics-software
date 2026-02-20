from dataclasses import dataclass, field
import random
import time
import logging
from labjack import ljm
from .models.sensor import Sensor

logger = logging.getLogger(__name__)


@dataclass
class LabjackIngestorService:
    device_type: str = field(default="T7")
    device_ip: str = field(default="192.168.0.250")
    handle: int = field(default=None)
    transaction: tuple = field(default_factory=list)
    mock: bool = field(default=False)

    def configure_sensor(self, sensor: Sensor):
        if self.mock:
            logger.info(
                "Mock mode enabled. Skipping sensor configuration for: %s", sensor.name
            )
            return
        logger.info("Configuring sensor: %s", sensor.name)
        self.write_register(
            f"AIN{sensor.positive_channel}_NEGATIVE_CH", sensor.negative_channel
        )
        self.write_register(f"AIN{sensor.positive_channel}_RANGE", sensor.range)

    def read_sensor(self, sensor: Sensor) -> float:
        try:
            if self.mock:
                time.sleep(0.1)  # Simulate read delay
                value = random.uniform(-sensor.range, sensor.range)
            else:
                value = ljm.eReadName(self.handle, f"AIN{sensor.positive_channel}")

            scaled_value = value * sensor.scale + sensor.offset
            return scaled_value
        except ljm.LJMError as e:
            logger.error("Read Error for %s: %s", sensor.name, e)
            return None

    def connect(self):
        if self.mock:
            logger.info("Mock mode enabled. Skipping Labjack connection.")
            return
        if self.handle:
            logger.info("Labjack is already connected. Skipping reconnection attempt")
            return
        try:
            logger.info("Connecting to Labjack...")
            self.handle = ljm.openS(self.device_type, "ETHERNET", self.device_ip)
            info = ljm.getHandleInfo(self.handle)
            logger.info(
                "Opened a LabJack with Device type: %i, Connection type: %i, Serial number: %i, IP address: %s, Port: %i,\nMax bytes per MB: %i",
                info[0],
                info[1],
                info[2],
                ljm.numberToIP(info[3]),
                info[4],
                info[5],
            )
        except ljm.LJMError as e:
            logger.error("Connection Error: %s", e)
            self.handle = None

    def disconnect(self):
        if self.mock:
            logger.info("Mock mode enabled. Skipping Labjack disconnection.")
            return
        try:
            ljm.close(self.handle)
            logger.info("Disconnected successfully.")
        except ljm.LJMError as e:
            logger.error("Disconnection Error: %s", e)
        finally:
            self.handle = None

    def start_transaction(self):
        self.transaction = ([], [])

    def write_register(self, name, value):
        if not self.transaction:
            ljm.eWriteName(self.handle, name, value)
            return
        self.transaction[0].append(name)
        self.transaction[1].append(value)

    def clear_transaction_list(self):
        self.transaction = None

    def commit_transaction(self):
        if self.mock:
            logger.info("Mock mode enabled. Skipping transaction commit.")
            self.clear_transaction_list()
            return
        if not self.transaction:
            logger.warning("No transaction to commit.")
            return
        try:
            ljm.eWriteNames(
                self.handle,
                len(self.transaction[0]),
                self.transaction[0],
                self.transaction[1],
            )
            logger.info("Transaction committed successfully.")
            sleep(0.1)  # Sleep for 100ms to allow the transaction to complete
        except ljm.LJMError as e:
            logger.error("Transaction Commit Error: %s", e)
        finally:
            self.clear_transaction_list()

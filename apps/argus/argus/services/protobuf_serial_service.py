import serial
from argus.envelope_pb2 import Envelope
import logging
from services.persistence_service import PersistenceService


class ProtobufSerialService:
    device: serial.Serial
    persistence_service: PersistenceService

    def __init__(
        self, port: str, baudrate: int, persistence_service: PersistenceService
    ):
        self.device = serial.Serial(
            port=port,
            baudrate=baudrate,
            bytesize=serial.EIGHTBITS,
            parity=serial.PARITY_NONE,
            stopbits=serial.STOPBITS_ONE,
            timeout=1.0,
        )
        self.persistence_service = persistence_service
        self.logger = logging.getLogger(ProtobufSerialService.__name__)
        self.logger.info("ProtobufSerialService initialized on port %s", port)

    def read_varint(self) -> int:
        """
        Read a protobuf-style varint from the serial port, one byte at a time.
        - Each byte contributes 7 bits of payload.
        - If the MSB (bit 7) is set, another varint byte follows.
        - Returns the decoded integer (e.g., a length or a type ID).
        - Raises TimeoutError if the serial port times out before we finish.
        - Raises ValueError if the varint is longer than 10 bytes (invalid).
        """
        shift = 0
        result = 0

        # Protobuf varints for 64-bit values take at most 10 bytes
        for _ in range(10):
            b = self.device.read(1)  # read exactly one byte from UART
            if not b:  # empty means serial timeout (per pyserial timeout)
                raise TimeoutError("Serial timeout while reading varint")

            byte = b[0]  # turn single-byte bytes -> int [0..255]
            result |= (
                byte & 0x7F
            ) << shift  # add 7 bits to our result at the current shift

            if (
                byte & 0x80
            ) == 0:  # MSB clear means this was the last byte of the varint
                return result

            shift += 7  # prepare to place the next 7 bits above the current ones

        # If we get here, we received 10 continuation bytes → malformed stream
        self.logger.error("Received malformed varint (too long)")
        raise ValueError("Varint too long (invalid stream)")

    def read_exact(self, n: int) -> bytes:
        """
        Read exactly n bytes from the serial port, blocking until they arrive or we time out.
        - Returns a bytes object of length n.
        - Raises TimeoutError if not enough bytes arrive before the port’s timeout.
        """
        buf = bytearray()
        while len(buf) < n:
            chunk = self.device.read(n - len(buf))  # ask for the remainder
            if not chunk:  # timeout or stream ended
                raise TimeoutError(f"Serial timeout while reading {n} payload bytes")
            buf.extend(chunk)
        return bytes(buf)

    def read_loop(self):
        while True:
            try:
                length = self.read_varint()
                payload = self.read_exact(length)
                envelope = Envelope()
                envelope.ParseFromString(payload)
                message_type = envelope.WhichOneof("message")
                if message_type:
                    try:
                        message = getattr(envelope, message_type)
                        self.persistence_service.store_protobuf(message)
                    except Exception as e:
                        self.logger.error(
                            "Could not store envelope. Error: %s; Envelope: %s",
                            repr(e),
                            envelope,
                        )
                else:
                    # If no oneof is set, we still got a valid Envelope, just empty
                    self.logger.warning("Envelope has no message set (unknown/empty)")

            except TimeoutError:
                # Benign: just means we didn’t receive a complete frame within timeout
                # Loop continues to try again.
                continue

            except Exception as e:
                # Any other error (malformed varint, parse error, etc.)
                # In a production system, consider:
                # - logging the error
                # - trying to resynchronize (e.g., read until a plausible varint appears)
                self.logger.error("Protobuf parsing error: %s", repr(e))

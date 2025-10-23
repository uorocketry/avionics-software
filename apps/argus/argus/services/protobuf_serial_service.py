import serial
from messages.argus.envelope_pb2 import Envelope
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

    def read_envelope(self) -> Envelope:
        length = self.read_varint()
        payload = self.read_exact(length)
        envelope = Envelope()
        envelope.ParseFromString(payload)
        return envelope

    def write_envelope(
        self,
        proto,
    ) -> None:
        """
        Send a protobuf message as:
            [varint length][protobuf bytes]
        """

        # Serialize protobuf to bytes
        payload = proto.SerializeToString()
        varint = self.encode_varint(len(payload))

        # Prefix with length varint
        frame = bytearray()
        frame += varint
        frame += payload

        # Ship it
        self.device.write(frame)
        self.device.flush()  # make sure it’s pushed to the wire

    def encode_varint(self, length: int) -> bytes:
        """Encode a non-negative integer as a protobuf varint."""
        out = bytearray()
        while length >= 0x80:
            out.append((length & 0x7F) | 0x80)
            length >>= 7
        out.append(length)
        return bytes(out)

import logging
from services.protobuf_serial_service import ProtobufSerialService
from services.persistence_service import PersistenceService


class MessageIngestionService:
    def __init__(
        self,
        protobuf_serial_service: ProtobufSerialService = None,
        persistence_service: PersistenceService = None,
    ):
        self.logger = logging.getLogger(MessageIngestionService.__name__)
        self.protobuf_serial_service = protobuf_serial_service
        self.persistence_service = persistence_service

    def ingest_loop(self):
        while True:
            try:
                envelope = self.protobuf_serial_service.read_envelope()
                message_type = envelope.WhichOneof("message")
                if message_type:
                    try:
                        message = getattr(envelope, message_type)
                        self.logger.debug(
                            "Received message of type %s: %s", message_type, message
                        )
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

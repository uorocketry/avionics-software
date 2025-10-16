import argparse
import utils.logger
from services.persistence_service import PersistenceService
from services.protobuf_serial_service import ProtobufSerialService
from services.session_service import SessionService

program = argparse.ArgumentParser(description="Argus Ground Station Application")

program.add_argument("port", type=str)
program.add_argument("--baudrate", type=int, default=115200)

if __name__ == "__main__":
    args = program.parse_args()
    session_service = SessionService()
    session_service.start_session()
    persistence_service = PersistenceService(session_service=session_service)
    protobuf_serial = ProtobufSerialService(
        port=args.port, baudrate=args.baudrate, persistence_service=persistence_service
    )
    protobuf_serial.read_loop()

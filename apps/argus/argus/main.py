import asyncio
import argparse
import contextlib
import utils.logger
from services.persistence_service import PersistenceService
from services.protobuf_serial_service import ProtobufSerialService
from services.grpc_service import GrpcService
from services.argus_service import ArgusService
from services.session_service import SessionService

# from argus.envelope_pb2 import Envelope
# from argus.temperature.thermocouple_calibration_pb2 import ThermocoupleCalibration

program = argparse.ArgumentParser(description="Argus Ground Station Application")

program.add_argument("port", type=str)
program.add_argument("--baudrate", type=int, default=115200)


async def main():
    args = program.parse_args()
    session_service = SessionService()
    session_service.start_session()
    persistence_service = PersistenceService(session_service=session_service)
    protobuf_serial_service = ProtobufSerialService(
        port=args.port, baudrate=args.baudrate, persistence_service=persistence_service
    )
    argus_service = ArgusService(protobuf_serial_service=protobuf_serial_service)
    grpc_service = GrpcService(services=[argus_service], port=50051)

    serial_task = asyncio.create_task(
        asyncio.to_thread(protobuf_serial_service.read_loop)
    )
    grpc_task = asyncio.create_task(grpc_service.serve())
    try:
        await grpc_task
    finally:
        serial_task.cancel()
        with contextlib.suppress(asyncio.CancelledError):
            await serial_task
        protobuf_serial_service.device.close()


if __name__ == "__main__":
    asyncio.run(main())

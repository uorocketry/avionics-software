import asyncio
import argparse
import contextlib
import utils.logger
import messages.argus
from services.persistence_service import PersistenceService
from services.protobuf_serial_service import ProtobufSerialService
from services.grpc_service import GrpcService
from services.argus_service import ArgusService
from services.message_ingestion_service import MessageIngestionService
from services.session_service import SessionService
from utils.database import database

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
    message_ingestion_service = MessageIngestionService(
        protobuf_serial_service=protobuf_serial_service,
        persistence_service=persistence_service,
    )
    argus_service = ArgusService(protobuf_serial_service=protobuf_serial_service)
    grpc_service = GrpcService(services=[argus_service], port=50051)

    database.evolve()

    ingestion_task = asyncio.create_task(
        asyncio.to_thread(message_ingestion_service.ingest_loop)
    )
    grpc_task = asyncio.create_task(grpc_service.serve())
    try:
        await grpc_task
    finally:
        ingestion_task.cancel()
        with contextlib.suppress(asyncio.CancelledError):
            await ingestion_task
        protobuf_serial_service.device.close()


if __name__ == "__main__":
    asyncio.run(main())

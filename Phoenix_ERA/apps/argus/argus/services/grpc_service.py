import grpc
import logging


class GrpcService:
    def __init__(self, port: int = 50051, services=None):
        self.port = port
        self.services = services or []
        self.server = None
        self.logger = logging.getLogger(GrpcService.__name__)

    async def serve(self):
        if self.server:
            self.server.stop(0)

        # The gRPC server must be created inside the running event loop.
        self.server = grpc.aio.server()
        for service in self.services:
            service.register(self.server)
        self.server.add_insecure_port(f"[::]:{self.port}")
        await self.server.start()
        self.logger.info("gRPC server started on port %s", self.port)
        await self.server.wait_for_termination()

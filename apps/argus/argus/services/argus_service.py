import argus.service_pb2_grpc
from google.protobuf import empty_pb2
from argus.envelope_pb2 import Envelope
from services.protobuf_serial_service import ProtobufSerialService


class ArgusService(argus.service_pb2_grpc.ArgusServicer):
    def __init__(self, protobuf_serial_service: ProtobufSerialService = None):
        self.protobuf_serial_service = protobuf_serial_service

    async def SendEnvelope(self, envelope: Envelope, context):
        self.protobuf_serial_service.write_envelope(envelope)
        return empty_pb2.Empty()

    def register(self, server):
        argus.service_pb2_grpc.add_ArgusServicer_to_server(self, server)

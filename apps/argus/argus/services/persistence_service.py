import logging
from services.session_service import SessionService
from dtos import message_type_to_model


class PersistenceService:
    def __init__(self, session_service: SessionService):
        self.session_service = session_service
        self.logger = logging.getLogger(PersistenceService.__name__)

    def transform_protobuf_to_model(self, proto):
        message_type = type(proto)
        model_class = message_type_to_model.get(message_type)
        if model_class is None:
            raise ValueError(f"No model class found for message type {message_type}")
        return model_class.from_protobuf(proto)

    def store_protobuf(self, proto):
        model = self.transform_protobuf_to_model(proto)
        model.host_session = self.session_service.active_session
        model.save()

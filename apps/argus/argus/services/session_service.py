import logging
from models.recording_session import HostRecordingSession


class SessionService:
    def __init__(self):
        self.active_session = None
        self.logger = logging.getLogger(SessionService.__name__)

    def start_session(self):
        self.active_session = HostRecordingSession.create()
        self.logger.info(f"Started new recording session %s", repr(self.active_session))

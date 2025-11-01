from peewee import Model, AutoField, TimestampField, CharField
from utils.database import database


class HostRecordingSession(Model):
    id = AutoField()
    name = CharField(max_length=255, null=True)
    start_time = TimestampField()

    class Meta:
        table_name = "host_recording_sessions"
        database = database

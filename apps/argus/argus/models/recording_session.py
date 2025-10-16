from peewee import Model, AutoField, TimestampField, CharField
from utils.database import database


class RecordingSession(Model):
    id = AutoField()
    name = CharField(max_length=255, null=True)
    start_time = TimestampField()

    class Meta:
        table_name = "recording_sessions"
        database = database


database.create_tables([RecordingSession])

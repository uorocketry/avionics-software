from peewee import Model, CharField, IntegerField, FloatField, AutoField
from utils.database import database


class Sensor(Model):
    id = AutoField()
    name = CharField(unique=True)
    positive_channel = IntegerField(default=0)  # AIN0
    negative_channel = IntegerField(default=1)  # AIN1 | Set to 199 for GND/Single Ended
    range = IntegerField(default=10)
    scale = FloatField(default=1.0)
    offset = FloatField(default=0.0)

    class Meta:
        database = database
        table_name = "sensors"
        schema = "phil_cart"

    def ensure_exists(self):
        existing = Sensor.get_or_none(Sensor.name == self.name)

        if existing is None:
            self.save(force_insert=True)  # insert new row
            return self

        # update existing row
        self.id = existing.id  # update id to match existing row
        existing.positive_channel = self.positive_channel
        existing.negative_channel = self.negative_channel
        existing.range = self.range
        existing.scale = self.scale
        existing.offset = self.offset
        existing.save()
        return existing

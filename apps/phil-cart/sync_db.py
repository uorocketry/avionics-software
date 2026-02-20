import peeweedbevolve
from utils.database import database

# Import all the models so that peeweedbevolve can find them
from daq.models.sensor import Sensor
from daq.models.datapoint import DataPoint

if __name__ == "__main__":
    database.execute_sql('CREATE SCHEMA IF NOT EXISTS "phil_cart"')
    database.evolve(schema="phil_cart")

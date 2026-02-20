from peewee import PostgresqlDatabase


database = PostgresqlDatabase(
    "uorocketry",
    user="admin",
    password="admin",
    host="localhost",
    port=5432,
)

import logging
import os

logging.basicConfig(level=os.getenv("LOG_LEVEL", logging.INFO))

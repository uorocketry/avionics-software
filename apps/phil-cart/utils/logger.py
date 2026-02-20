import logging
from colorlog import ColoredFormatter
import os

formatter = ColoredFormatter(
    "[%(log_color)s%(asctime)s][%(levelname)-8s][%(name)s] %(message)s",
    datefmt="%H:%M:%S",
    log_colors={
        "DEBUG": "gray",
        "INFO": "cyan",
        "WARNING": "yellow",
        "ERROR": "red",
        "CRITICAL": "bold_red",
    },
)

handler = logging.StreamHandler()
handler.setFormatter(formatter)

root_logger = logging.getLogger()
root_logger.setLevel(os.getenv("LOG_LEVEL", logging.INFO))

# Remove existing handlers to avoid duplicates
if root_logger.hasHandlers():
    root_logger.handlers.clear()

root_logger.addHandler(handler)

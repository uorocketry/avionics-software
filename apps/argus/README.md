# Argus Ground Station Software

Currently just interfacing with the Argus board using UART. You need a UART TTY to USB adapter to use this software.

## Setup
Make sure you have `uv` installed: `pipx install uv` 

## Running
Find out which port your UART-to-USB adapter is mounted on and run `uv run main.py <port>`. 


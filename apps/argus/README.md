# Argus Ground Station Software

Ground station to collect data from argus and store in a database. 
Currently just interfacing with the Argus board using UART. You need a UART TTY to USB adapter to use this software.

## Setup
Make sure you have `uv` installed: `pipx install uv` 

## Running
Ensure `grafana` and `persistence` services are running by calling `docker compose up -d` within their respective directories.

Find out which port your UART-to-USB adapter is mounted on and run the argus service by calling `uv run main.py <port>`. 

Once argus is connected and you're seeing logs of data stream coming in, you can open up grafana at `http://localhost:3000` and import one of the dashboards from `apps/grafana/dashboards` that is configured to connect to the `postgres` service defined in `persistence/docker-compose.yml` and you can see the logs coming in. 

You need to input the session you'd like to see in the grafana dashboard. Every time that the argus service is restarted a new session is created. This is to be able to jump to specific historical sessions easily.
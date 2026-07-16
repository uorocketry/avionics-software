#!/bin/bash

# Script to keep trying to connect to an Xbox controller using bluetoothctl

# Replace this with your Xbox controller's MAC address
XBOX_MAC="C8:3F:26:3D:28:D9"

if [ -z "$XBOX_MAC" ] || [ "$XBOX_MAC" = "XX:XX:XX:XX:XX:XX" ]; then
    echo "Please set your Xbox controller's MAC address in the script (XBOX_MAC variable)."
    exit 1
fi

echo "Attempting to connect to Xbox controller at $XBOX_MAC..."

while true; do
    # Check if the controller is already connected
    connected=$(bluetoothctl info "$XBOX_MAC" | grep "Connected: yes")
    if [ -n "$connected" ]; then
        echo "Controller is connected."
    else
        echo "Controller not connected. Trying to connect..."
        bluetoothctl connect "$XBOX_MAC"
    fi
    # Wait 5 seconds before trying again
    sleep 5
done

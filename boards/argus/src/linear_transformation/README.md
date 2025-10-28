# Linear Transformation 
This service handles loading/storing linear transformations that are applied to any reading. (Could be temperature, pressure, strain, etc.)

It takes some form of data persistence service (currently only `sd_card_service` supported), and creates a file to which it reads/writes the scale and offset that needs to be applied for a given device and its channels.


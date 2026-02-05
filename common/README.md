## High-Level
`common\high-level` Contains all high-level (abstract) structures & logic. <br> <Br> 
Examples of high-level (highly-abstract) logic includes: 
1.  State Machines
2.  Communication Managers
3.  Anything that is a "wraps" around a driver
  
## Drivers
`common\drivers` Contains all device & ic drivers (software that controls physical things, but not peripherals). <br> <br> 
Examples of drivers include: 
1.  Driver for the RFD900X
2.  Driver for Barometer (MS561101)
3.  etc.
  
## Peripherals
`common\peripherals` Contains all peripheral controllers/drivers (software that controls the peripherals).This exists to add further functionality & abstraction to the existing HAL Embassy provides. <br> <br> 

Examples of drivers include: 
1.  SPI driver
2.  CAN driver (using the MCU component, NOT the IC/PHY)
3.  USB driver (using the MCU component, NOT THE PHY)
4.  I2C driver
5.  UART driver
  
## UOR Utils
`common\uor-utils` Contains all utilities that are agnostic (they don't care who or what uses them).

Agnostic utilities include: 
1.  Filters
2.  Math tools & Units
3.  Types/Traits unrelated to peripherals, drivers, or high-level logic
4.  Macros unrelated to peripherals, drivers, or high-level logic

<br>

NON-Agnostic utilities (these belong in their respective crates/modules) include: 
1. Types/Traits related to peripherals, drivers, or high-level logic.
2. Configuration data
3. Macros  related to peripherals, drivers, or high-level logic.

## UOR Proc Macros
`common\uor-proc-macros` Contains all procedural macros used elsewhere. This crate exists because: 
1. Procedural macros must be built in a Std environment, and all other crates are no_std. x
2. Proc macros also require their crates to only export proc macros, nothing else.
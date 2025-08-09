#![feature(impl_trait_in_assoc_type)]
#![no_std]
#![no_main]

#[cfg(not(any(feature = "pressure", feature = "temperature", feature = "strain")))]
compile_error!(
    "You must enable exactly one of the features: 'pressure', 'temperature', or 'strain'."
);

mod traits;
mod adc_manager;

use embassy_stm32::exti::ExtiInput;
use libm::{logf, powf};
use core::cell::RefCell;
use core::marker::PhantomData;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_stm32::gpio::{Input, Level, Output, OutputType, Pull, Speed};
use embassy_stm32::mode::Blocking;
use embassy_stm32::rtc::Rtc;
use embassy_stm32::spi::{BitOrder, Config as SpiConfig, Spi};
use embassy_stm32::time::{hz, khz, mhz};
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::usart::{Config as UartConfig, RingBufferedUartRx, Uart, UartTx};
use embassy_stm32::{bind_interrupts, mode, peripherals, usart};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::channel::Channel;
use embassy_time::{Delay, Duration, Instant, Ticker, Timer};
use embedded_alloc::Heap;
use heapless::HistoryBuffer;
use messages_prost::sensor::sbg::SbgData;
use static_cell::StaticCell;
use crate::traits::Context;
use {defmt_rtt as _};
use {panic_probe as _};
use embedded_sdmmc::{Mode, SdCard, VolumeIdx, VolumeManager};
use embedded_hal_bus::spi::RefCellDevice;
use pid::Pid;

// Use the asynchronous SpiDevice from embassy-embedded-hal

use smlang::statemachine;

// --- System Configuration ---

/// The target temperature we want to maintain.
const SETPOINT_TEMP_C: f32 = 25.0;

/// How often the PID control loop runs.
const CONTROL_LOOP_INTERVAL_MS: u64 = 1000;

// The maximum raw value for the ADC (2^12 - 1 for a 12-bit ADC).
const ADC_MAX_VALUE: f32 = 4095.0;

// The value of the fixed resistor in your voltage divider circuit (in Ohms).
// This is taken from the schematic (R5 = 1.6kΩ).
const DIVIDER_RESISTANCE: f32 = 1600.0;


// --- NTC Thermistor Datasheet Parameters ---
// IMPORTANT: You MUST get these values from the datasheet for YOUR specific thermistor.
// These are common values for a standard 10k NTC thermistor.

/// Nominal resistance at the nominal temperature (e.g., 10kΩ at 25°C).
const THERMISTOR_NOMINAL_RESISTANCE: f32 = 10000.0;

/// The Beta coefficient of the thermistor (often in the range 3000-4500).
const THERMISTOR_BETA: f32 = 3950.0;

/// Nominal temperature in Kelvin (25°C).
const TEMPERATURE_NOMINAL_KELVIN: f32 = 298.15; // 25.0 + 273.15


// --- PID Tuning Constants ---
// You MUST tune these for your specific hardware setup.
const KP: f32 = 2.5;
const KI: f32 = 0.1;
const KD: f32 = 0.5;

// =================================================================================
// Shared Resources & Types
// =================================================================================

#[global_allocator]
static HEAP: Heap = Heap::empty();

// static FAULT_CHANNEL: Channel<CriticalSectionRawMutex, , 2> = Channel::new();

// The SPI bus is protected by a Mutex, so the RefCell is not needed.
static SPI_BUS: StaticCell<embassy_sync::mutex::Mutex<CriticalSectionRawMutex, Spi<mode::Async>>> = StaticCell::new();

// Static variable for the RTC
pub static RTC: Mutex<CriticalSectionRawMutex, RefCell<Option<Rtc>>> =
    Mutex::new(RefCell::new(None));

bind_interrupts!(struct Irqs {
    UART7 => usart::InterruptHandler<peripherals::UART7>;
    UART8 => usart::InterruptHandler<peripherals::UART8>;
});

statemachine! {
    transitions: {
        *Init + Start = WaitForLaunch,
        WaitForLaunch + Launch = Ascent,
        Ascent + Apogee = Descent,
        Descent + MainDeployment = Fuck, 
        Descent + DrogueDeployment = DrogueDescent, 
        DrogueDescent + MainDeployment =  MainDescent,
        MainDescent + NoMovement = Landed,
        Fault + FaultCleared = _,
        _ + FaultDetected = Fault,
    }
}

pub struct TimeSink {
    _marker: PhantomData<*const ()>,
}

impl TimeSink {
    fn new() -> Self {
        TimeSink {
            _marker: PhantomData,
        }
    }
}

impl embedded_sdmmc::TimeSource for TimeSink {
    fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
        embedded_sdmmc::Timestamp {
            year_since_1970: 0,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}

// =================================================================================
// Application Tasks
// =================================================================================

#[embassy_executor::task]
async fn led_blinker_task(pin: peripherals::PA3) {
    let mut led = Output::new(pin, Level::High, Speed::Low);
    info!("LED blinker task started.");
    loop {
        led.set_high();
        Timer::after_millis(500).await;
        led.set_low();
        Timer::after_millis(500).await;
    }
}


/// Converts a raw ADC reading from the thermistor's voltage divider
/// into a temperature in Celsius.
fn adc_to_celsius(adc_value: u16) -> f32 {
    // 1. Calculate the resistance of the thermistor using the voltage divider formula.
    // This formula works regardless of the input voltage (3.3V or 5V) as it's ratiometric.
    // R_thermistor = R_fixed / ((ADC_MAX / ADC_reading) - 1)
    let resistance = DIVIDER_RESISTANCE / ((ADC_MAX_VALUE / adc_value as f32) - 1.0);

    // 2. Calculate temperature using the Beta-parameter equation.
    // 1/T = 1/T0 + (1/B) * ln(R/R0)
    let steinhart = logf(resistance / THERMISTOR_NOMINAL_RESISTANCE) / THERMISTOR_BETA
        + (1.0 / TEMPERATURE_NOMINAL_KELVIN);
    
    let temp_kelvin = 1.0 / steinhart;

    // 3. Convert from Kelvin to Celsius.
    let temp_celsius = temp_kelvin - 273.15;

    temp_celsius
}

/// Sets the heater state based on the PID controller's output.
/// The PID output is treated as a percentage. If it's over a threshold,
/// the heater turns on, otherwise it turns off. This is a simple
/// way to use a PID controller for on/off (bang-bang) control.
fn set_heater_state(
    heater_pin: &mut Output,
    pid_output: f32,
) {
    // We can use a simple threshold. If the PID controller requests more than
    // 50% power, we turn the heater on. Otherwise, we turn it off.
    // This threshold can be adjusted. A lower threshold will make the
    // heater turn on more readily.
    if pid_output > 50.0 {
        heater_pin.set_high();
    } else {
        heater_pin.set_low();
    }
}

#[embassy_executor::task]
async fn temperature_regulator(
    mut adc: Adc<'static, embassy_stm32::peripherals::ADC1>,
    mut temp_pin: embassy_stm32::peripherals::PB1,
    mut heater_pin: Output<'static>,
) {
    defmt::info!("Temperature regulator task started.");
    
    // Configure the PID controller.
    let mut pid = Pid::new(SETPOINT_TEMP_C, 100.0);
    pid.p(KP, 100.0)
       .i(KI, 100.0) // Limit integral contribution to prevent wind-up
       .d(KD, 100.0);

    let mut ticker = Ticker::every(Duration::from_millis(CONTROL_LOOP_INTERVAL_MS));

    loop {
        // Read the raw ADC value from the thermistor pin.
        let adc_raw = adc.blocking_read(&mut temp_pin);
        
        // Convert the raw value to a temperature in Celsius.
        let measurement = adc_to_celsius(adc_raw);

        // Calculate the new control output.
        let control_output = pid.next_control_output(measurement);
        
        // Apply the new output to the heater pin (on/off).
        set_heater_state(&mut heater_pin, control_output.output);

        defmt::info!(
            "Setpoint: {}°C, Measured: {}°C -> PID Output: {} (P: {}, I: {}, D: {}) -> Heater: {}",
            SETPOINT_TEMP_C,
            measurement,
            control_output.output,
            control_output.p,
            control_output.i,
            control_output.d,
            if heater_pin.is_set_high() { "ON" } else { "OFF" }
        );

        // Wait for the next tick.
        ticker.next().await;
    }
}

#[embassy_executor::task] 
async fn sm_task(spawner: Spawner, state_machine: StateMachine<Context>) {
    info!("State Machine task started.");

    loop {
        match state_machine.state {
            States::Ascent => {

            },
            States::Fault => {

            },
            States::Init => {
                
            },
            States::WaitForLaunch => {
                
            },
            States::Descent => {

            },
            States::DrogueDescent => {

            },
            States::Fuck => {

            },
            States::Landed => {

            },
            States::MainDescent => {

            }
        } 
        Timer::after(Duration::from_millis(1000)).await;
    }
}

// =================================================================================
// Main Entry Point
// =================================================================================

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("System starting...");
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 40000;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    }

    let mut config = embassy_stm32::Config::default();

    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::DIV1);
        config.rcc.csi = true;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV2),
            divq: Some(PllDiv::DIV8), // used by SPI3. 100Mhz.
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
    }
    // config.rcc.ls = rcc::LsConfig::default_lse();
    let p = embassy_stm32::init(config);

    // --- ADS 126 Setup --- 
    let mut adc_spi_config = SpiConfig::default(); 
    adc_spi_config.frequency = mhz(8); 
    adc_spi_config.mode = embassy_stm32::spi::MODE_1; 

    let adc_spi_bus = Spi::new_blocking(p.SPI4, p.PE2, p.PE6, p.PE5, adc_spi_config);

    let mut adc1_cs = Output::new(p.PE1, Level::High, Speed::Low);

    let mut adc2_cs = Output::new(p.PB8, Level::High, Speed::Low);

    let mut adc1_rst = Output::new(p.PE0, Level::High, Speed::Low);
    let mut adc2_rst = Output::new(p.PB7, Level::High, Speed::Low);

    let mut adc1_drdy = ExtiInput::new(p.PB9, p.EXTI9, Pull::Down);
    let mut adc2_drdy = ExtiInput::new(p.PB6, p.EXTI6, Pull::Down);

    let mut adc_manager = adc_manager::AdcManager::new(
        adc_spi_bus, 
        adc1_rst,
        adc2_rst,
        adc1_cs,
        adc2_cs,
    );

    adc_manager.init_adc1().unwrap();
    adc_manager.init_adc2().unwrap();



    #[cfg(feature = "pressure")]
    {
        adc_manager.set_adc2_inpmux(
            ads126x::register::NegativeInpMux::AIN6,
            ads126x::register::PositiveInpMux::AIN7,
        );
    }
    #[cfg(feature = "temperature")]
    {
        adc_manager.set_adc1_inpmux(
            ads126x::register::NegativeInpMux::AIN1,
            ads126x::register::PositiveInpMux::AIN0,
        );
        adc_manager.set_adc2_inpmux(
            ads126x::register::NegativeInpMux::AIN0,
            ads126x::register::PositiveInpMux::AIN1,
        );
    }
    loop {
        if let Ok(data) = adc_manager.read_adc1_data() {
            info!("ADC1 Data: {:?}", data);
            #[cfg(feature = "temperature")]
            {
                let volts = thermocouple_converter::adc_to_voltage(data);
                info!("volatage: {}", volts);

                let celsius = thermocouple_converter::adc_to_celsius(data);
                info!("Celcius: {}", celsius);
            }

            #[cfg(feature = "pressure")]
            {
                let volts = thermocouple_converter::adc_to_voltage(data);
                info!("volatage: {}", volts);
                let pressure: f64 = ((10000.0/((60.0/100.0) * (2.5 / 3.0))) * volts) / 32.0;
                info!("Pressure (psi): {}", pressure);
            }

            #[cfg(feature = "strain")]
            {
                let volts = thermocouple_converter::adc_to_voltage(data);
                info!("volatage: {}", volts);
                let strain = data as f32 * 0.0001;
                info!("Strain: {}", strain);
            }
        } else {
            info!("Failed to read ADC1 data.");
        }
        if let Ok(data) = adc_manager.read_adc2_data() {
            info!("ADC2 Data: {:?}", data);
            #[cfg(feature = "temperature")]
            {
                let volts = thermocouple_converter::adc_to_voltage(data);
                info!("volatage: {}", volts);

                let celsius = thermocouple_converter::adc_to_celsius(data);
                info!("Celcius: {}", celsius);
            }

            #[cfg(feature = "pressure")]
            {
                let volts = thermocouple_converter::adc_to_voltage(data);
                info!("volatage: {}", volts);
                let pressure: f64 = ((10000.0/((60.0/100.0) * (2.5 / 3.0))) * volts) / 32.0;
                info!("Pressure (psi): {}", pressure);
            }

            #[cfg(feature = "strain")]
            {
                let volts = thermocouple_converter::adc_to_voltage(data);
                info!("volatage: {}", volts);
                let strain = data as f32 * 0.0001;
                info!("Strain: {}", strain);
            }
        } else {
            info!("Failed to read ADC1 data.");
        }



        Timer::after(Duration::from_millis(1000)).await;
    }

    // --- SD Card ---
    let mut sd_spi_config = SpiConfig::default();

    sd_spi_config.frequency = mhz(16);
    
    sd_spi_config.mode = embassy_stm32::spi::Mode {
        polarity: embassy_stm32::spi::Polarity::IdleLow,
        phase: embassy_stm32::spi::Phase::CaptureOnFirstTransition,
    };

    sd_spi_config.bit_order = BitOrder::MsbFirst;

    let sd_spi_bus = Spi::new(
        p.SPI1, p.PA5, p.PA7, p.PA6, p.DMA1_CH4, p.DMA1_CH5, sd_spi_config,
    );

    let sd_cs = Output::new(p.PC4, Level::High, Speed::Low);

    let sd_spi_bus_ref_cell = RefCell::new(sd_spi_bus);
    let sd_spi_device = RefCellDevice::new(&sd_spi_bus_ref_cell, sd_cs, Delay);

    let sdcard = SdCard::new(sd_spi_device.unwrap(), Delay);
    println!("Card size is {} bytes", sdcard.num_bytes().unwrap());
    let volume_mgr = VolumeManager::new(sdcard, TimeSink::new());
    let volume0 = volume_mgr.open_volume(VolumeIdx(0)).unwrap();
    let root_dir = volume0.open_root_dir().unwrap();
    // let my_file = root_dir.open_file_in_dir("MY_FILE.TXT", Mode::ReadOnly).unwrap();
    // while !my_file.is_eof() {
    //     let mut buffer = [0u8; 32];
    //     let num_read = my_file.read(&mut buffer).unwrap();
    //     for b in &buffer[0..num_read] {
    //         info!("{}", *b as char);
    //     }
    // }
    // info!("Sd write and setup complete");

    // --- State Machine ---
    let state_machine = StateMachine::new(traits::Context {});

    #[cfg(feature = "pressure")]
    {
        // --- Heater Pin Setup ---
        // This is the single pin that controls the heater.
        let heater_pin = Output::new(p.PE11, Level::Low, Speed::Low);
        let mut adc = Adc::new(p.ADC1);
        adc.set_sample_time(SampleTime::CYCLES32_5);
        let temp_pin = p.PB1; // Your thermistor pin
        spawner.spawn(temperature_regulator(adc, temp_pin, heater_pin)).unwrap();
    }    

    // NOTE 
    // Creating multiple executor instances is supported, to run tasks with multiple priority levels. This allows higher-priority tasks to preempt lower-priority tasks.

    // --- Spawning Tasks ---
    spawner.must_spawn(led_blinker_task(p.PA3));

    // Spawn the regulator task, passing it the hardware peripherals.
    // pass control of the spawner to the state machine
    spawner.must_spawn(sm_task(spawner, state_machine));
}
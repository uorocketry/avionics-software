#![no_std]
#![no_main]

#[cfg(not(any(feature = "pressure", feature = "temperature", feature = "strain")))]
compile_error!(
    "You must enable exactly one of the features: 'pressure', 'temperature', or 'strain'."
);

use argus::*;

use adc_manager::AdcManager;
use chrono::NaiveDate;
use common_arm::*;
use data_manager::DataManager;
use defmt::info;
use messages::CanData;
use messages::CanMessage;
use panic_probe as _;
use rtic_monotonics::systick::prelude::*;
use rtic_sync::{channel::*, make_channel};
use state_machine as sm;
use stm32h7xx_hal::gpio::gpioa::{PA2, PA3};
use stm32h7xx_hal::gpio::PA4;
use stm32h7xx_hal::gpio::{Edge, ExtiPin, Pin};
use stm32h7xx_hal::gpio::{Output, PushPull};
use stm32h7xx_hal::hal::spi;
use stm32h7xx_hal::prelude::*;
use stm32h7xx_hal::rtc;
use stm32h7xx_hal::{rcc, rcc::rec};
use types::COM_ID; // global logger
use core::marker::PhantomData;
use tokio::time::{interval, Duration}; // for time delays

use crate::types::{ADC2_RST_PIN_ID, ADC2_RST_PIN_PORT};

const DATA_CHANNEL_CAPACITY: usize = 10;

systick_monotonic!(Mono, 500);

const CALIBRATION_THRESHOLD: f32 = 0.1; //could change later
const MOTION_THRESHOLD: f32 = 2.0;

enum ImuWrapper{
    Uninitialized(Imu<Uninitialized>),
    Idling(Imu<Idle>),
    Calibrating(Imu<Calibrating>),
    Collecting(Imu<Collecting>),
    Entered_Fault(Imu<Entering_Fault>)

}

//defining state change
impl ImuWrapper {
    pub fn change<F>(&mut self, closure: F)
    where
        F: FnOnce(Self) -> Self,
    {
        unsafe {
            replace_with::replace_with_or_abort_unchecked(self, closure);
        }
    }
}

#[inline(never)]
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

#[rtic::app(device = stm32h7xx_hal::stm32, peripherals = true, dispatchers = [EXTI0, EXTI2, SPI3, SPI2])]
mod app {
    use messages::FormattedNaiveDateTime;

    use crate::time_manager::TimeManager;

    use super::*;

    #[shared]
    struct SharedResources {
        imu_wrapper: ImuWrapper,
        data_manager: DataManager,
        em: ErrorManager,
        sd_manager: SdManager<
            stm32h7xx_hal::spi::Spi<stm32h7xx_hal::pac::SPI1, stm32h7xx_hal::spi::Enabled>,
            PA4<Output<PushPull>>,
        >,
        // can_command_manager: CanManager<stm32h7xx_hal::can::Can<stm32h7xx_hal::pac::FDCAN1>>,
        // can_data_manager: CanManager<stm32h7xx_hal::can::Can<stm32h7xx_hal::pac::FDCAN2>>,
        rtc: rtc::Rtc,
        adc_manager: AdcManager<Pin<ADC2_RST_PIN_PORT, ADC2_RST_PIN_ID, Output<PushPull>>>,
        time_manager: TimeManager,
        state_machine: sm::StateMachine<traits::Context>,
    }

    #[local]
    struct LocalResources {
        can_sender: Sender<'static, CanMessage, DATA_CHANNEL_CAPACITY>,
        led_red: PA2<Output<PushPull>>,
        led_green: PA3<Output<PushPull>>,
        adc1_int: Pin<'A', 15, stm32h7xx_hal::gpio::Input>,
        adc2_int: Pin<'D', 3, stm32h7xx_hal::gpio::Input>,
    }

    #[init]
    fn init(ctx: init::Context) -> (SharedResources, LocalResources) {
        // channel setup
        let (can_sender, can_receiver) = make_channel!(CanMessage, DATA_CHANNEL_CAPACITY);

        let core = ctx.core;

        /* Logging Setup */
        // turn off logging for the moment
        // HydraLogging::set_ground_station_callback(queue_log_message);

        let pwr = ctx.device.PWR.constrain();
        // We could use smps, but the board is not designed for it
        // let pwrcfg = example_power!(pwr).freeze();
        let mut pwrcfg = pwr.freeze();

        info!("Power enabled");
        let backup = pwrcfg.backup().unwrap();
        info!("Backup domain enabled");
        // RCC
        let mut rcc = ctx.device.RCC.constrain();
        let reset = rcc.get_reset_reason();
        let fdcan_prec_unsafe = unsafe { rcc.steal_peripheral_rec() }
            .FDCAN
            .kernel_clk_mux(rec::FdcanClkSel::Pll1Q);

        let ccdr = rcc
            // .use_hse(48.MHz()) // check the clock hardware
            .sys_ck(96.MHz())
            .pll1_strategy(rcc::PllConfigStrategy::Iterative)
            .pll1_q_ck(48.MHz())
            .pclk1(48.MHz())
            .pclk2(48.MHz())
            .pclk3(48.MHz())
            .pclk4(48.MHz())
            .freeze(pwrcfg, &ctx.device.SYSCFG);
        info!("RCC configured");
        let fdcan_prec = ccdr
            .peripheral
            .FDCAN
            .kernel_clk_mux(rec::FdcanClkSel::Pll1Q);

        // GPIO
        let gpioa = ctx.device.GPIOA.split(ccdr.peripheral.GPIOA);
        let gpiod = ctx.device.GPIOD.split(ccdr.peripheral.GPIOD);
        let gpioc = ctx.device.GPIOC.split(ccdr.peripheral.GPIOC);
        let gpiob = ctx.device.GPIOB.split(ccdr.peripheral.GPIOB);
        let gpioe = ctx.device.GPIOE.split(ccdr.peripheral.GPIOE);

        // assert_eq!(ccdr.clocks.pll1_q_ck().unwrap().raw(), 32_000_000);
        info!("PLL1Q:");
        // https://github.com/stm32-rs/stm32h7xx-hal/issues/369 This needs to be stolen. Grrr I hate the imaturity of the stm32-hal
        // let can2: fdcan::FdCan<
        //     stm32h7xx_hal::can::Can<stm32h7xx_hal::pac::FDCAN2>,
        //     fdcan::ConfigMode,
        // > = {
        //     let rx = gpiob.pb12.into_alternate().speed(Speed::VeryHigh);
        //     let tx = gpiob.pb13.into_alternate().speed(Speed::VeryHigh);
        //     ctx.device.FDCAN2.fdcan(tx, rx, fdcan_prec)
        // };

        // let can_data_manager = CanManager::new(can2);

        // let can1: fdcan::FdCan<
        //     stm32h7xx_hal::can::Can<stm32h7xx_hal::pac::FDCAN1>,
        //     fdcan::ConfigMode,
        // > = {
        //     let rx = gpioa.pa11.into_alternate().speed(Speed::VeryHigh);
        //     let tx = gpioa.pa12.into_alternate().speed(Speed::VeryHigh);
        //     ctx.device.FDCAN1.fdcan(tx, rx, fdcan_prec_unsafe)
        // };

        // let can_command_manager = CanManager::new(can1);

        let spi_sd: stm32h7xx_hal::spi::Spi<
            stm32h7xx_hal::stm32::SPI1,
            stm32h7xx_hal::spi::Enabled,
            u8,
        > = ctx.device.SPI1.spi(
            (
                gpioa.pa5.into_alternate::<5>(), // sck
                gpioa.pa6.into_alternate(),      // miso
                gpioa.pa7.into_alternate(),      // mosi
            ),
            stm32h7xx_hal::spi::Config::new(spi::MODE_0),
            16.MHz(),
            ccdr.peripheral.SPI1,
            &ccdr.clocks,
        );

        let cs_sd = gpioa.pa4.into_push_pull_output();

        let sd_manager = SdManager::new(spi_sd, cs_sd);

        // ADC setup
        let adc_spi: stm32h7xx_hal::spi::Spi<
            stm32h7xx_hal::stm32::SPI4,
            stm32h7xx_hal::spi::Enabled,
            u8,
        > = ctx.device.SPI4.spi(
            (
                gpioe.pe2.into_alternate(),
                gpioe.pe5.into_alternate(),
                gpioe.pe6.into_alternate(),
            ),
            stm32h7xx_hal::spi::Config::new(spi::MODE_1), // datasheet mentioned a mode 1 per datasheet
            8.MHz(),                                      // 125 ns
            ccdr.peripheral.SPI4,
            &ccdr.clocks,
        );

        let adc1_cs = gpioc.pc10.into_push_pull_output();
        let adc2_cs = gpiod.pd2.into_push_pull_output();

        let adc1_rst = gpioc.pc11.into_push_pull_output();

        #[cfg(feature = "temperature")]
        let adc2_rst = gpioe.pe0.into_push_pull_output();

        #[cfg(feature = "pressure")]
        let adc2_rst = gpiod.pd1.into_push_pull_output();

        #[cfg(feature = "strain")]
        let adc2_rst = gpiob.pb9.into_push_pull_output();

        let mut adc_manager = AdcManager::new(adc_spi, adc1_rst, adc2_rst, adc1_cs, adc2_cs);
        adc_manager.init_adc1().ok();

        // leds
        let led_red = gpioa.pa2.into_push_pull_output();
        let led_green = gpioa.pa3.into_push_pull_output();

        let mut rtc = stm32h7xx_hal::rtc::Rtc::open_or_init(
            ctx.device.RTC,
            backup.RTC,
            stm32h7xx_hal::rtc::RtcClock::Lsi,
            &ccdr.clocks,
        );

        // TODO: Get current time from some source, this should be the responsibility of pheonix to sync the boards with GPS time.
        let now = NaiveDate::from_ymd_opt(2001, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        rtc.set_date_time(now.clone());

        let time_manager = TimeManager::new(Some(FormattedNaiveDateTime(now)));

        let mut syscfg = ctx.device.SYSCFG;
        let mut exti = ctx.device.EXTI;
        // setup interupt drdy pins
        let mut adc1_int = gpioa.pa15.into_pull_down_input();
        adc1_int.make_interrupt_source(&mut syscfg);
        adc1_int.trigger_on_edge(&mut exti, Edge::Rising);
        adc1_int.enable_interrupt(&mut exti);

        let mut adc2_int = gpiod.pd3.into_pull_down_input();
        adc2_int.make_interrupt_source(&mut syscfg);
        adc2_int.trigger_on_edge(&mut exti, Edge::Rising);
        adc2_int.enable_interrupt(&mut exti);

        /* Monotonic clock */
        Mono::start(core.SYST, 200_000_000);

        let mut data_manager = DataManager::new();
        data_manager.set_reset_reason(reset);
        let em = ErrorManager::new();
        let state_machine = sm::StateMachine::new(traits::Context {});

        blink::spawn().ok();
        // send_data_internal::spawn(can_receiver).ok();
        reset_reason_send::spawn().ok();
        state_send::spawn().ok();
        sm_orchestrate::spawn().ok();
        info!("Online");

        (
            SharedResources {
                imu_wrapper,
                data_manager,
                em,
                sd_manager,
                // can_command_manager,
                // can_data_manager,
                rtc,
                adc_manager,
                time_manager,
                state_machine,
            },
            LocalResources {
                adc1_int,
                adc2_int,
                can_sender,
                led_red,
                led_green,
            },
        )
    }

    /// The state machine orchestrator.
    /// Handles the current state of the ARGUS system.
    #[task(priority = 2, shared = [&state_machine])]
    async fn sm_orchestrate(cx: sm_orchestrate::Context) {
        let mut last_state = cx.shared.state_machine.state();
        loop {
            let state = cx.shared.state_machine.state();
            if state != last_state {
                _ = match state {
                    sm::States::Calibration => spawn!(sm_calibrate),
                    sm::States::Collection => spawn!(sm_collect),
                    sm::States::Fault => spawn!(sm_fault),
                    sm::States::Idle => spawn!(sm_idle),
                    sm::States::Init => spawn!(sm_init),
                };
                
                last_state = state;
            }

            Mono::delay(100.millis()).await;
        }
    }

    //generic imu state initialization
    pub struct Imu<State> {
        pub adc: AdcManager<Pin<'C', 11, Output<PushPull>>>,
        _state: PhantomData<State>,
    }

    pub struct Uninitialized;
    pub struct Idle;
    pub struct Calibrating;
    pub struct Collecting;
    pub struct EnteringFault;

    // Start from Uninitialized → Idle
impl Imu<Uninitialized> {
    pub fn initial_state(self) -> Imu<Idle> {
        Imu {
            adc: self.adc,
            _state: PhantomData,
        }
    }
}

// Idle → Calibrating or Collecting
impl Imu<Idle> {
    pub fn to_calibration(self) -> Result<Imu<Calibrating>, Imu<EnteringFault>> {
        Ok(Imu {
            adc: self.adc,
            _state: PhantomData,
        })
    }

    pub fn to_collection(self) -> Result<Imu<Collecting>, Imu<EnteringFault>> {
        Ok(Imu {
            adc: self.adc,
            _state: PhantomData,
        })
    }
}

// Calibrating → Idle
impl Imu<Calibrating> {
    pub fn finish_calibration(self) -> Result<Imu<Idle>, Imu<EnteringFault>> {
        Ok(Imu {
            adc: self.adc,
            _state: PhantomData,
        })
    }
}

// Collecting → Idle
impl Imu<Collecting> {
    pub fn done_collecting(self) -> Result<Imu<Idle>, Imu<EnteringFault>> {
        Ok(Imu {
            adc: self.adc,
            _state: PhantomData,
        })
    }
}


// Fault → Idle
impl Imu<EnteringFault> {
    pub fn exit_fault(self) -> Imu<Idle> {
        Imu {
            adc: self.adc,
            _state: PhantomData,
        }
    }
}



    #[task(priority = 3, shared = [imu_wrapper, state_machine, adc_manager, data_manager, em])]
    async fn sm_calibrate(cx: sm_calibrate::Context) {
        #[cfg(feature = "temperature")]
        {

        }

        #[cfg(feature = "pressure")]
         {

         }

        #[cfg(feature = "strain")]
         {

         }


        cx.shared.em.run(|| {
        
            (cx.shared.imu_wrapper, cx.shared.adc_manager, cx.shared.data_manager, cx.shared.state_machine).lock(|imu_wrapper, adc_manager, data_manager, state_machine| {
                match imu_wrapper {
                    ImuWrapper::Calibrating(imu) => {
                    
                        match adc_manager.read_adc1_data() {
                            Ok(accel_data) => {
                                match adc_manager.read_adc2_data() {
                                    Ok(gyro_data) => {
                                        
                                        info!("Calibrating with accel: {:?}, gyro: {:?}", accel_data, gyro_data);
                                        
                                       
                                        data_manager.add_calibration_sample(accel_data, gyro_data);
                                        
                                        let sample_count = data_manager.get_calibration_sample_count();
                                        if sample_count == 1 {
                                            data_manager.init_calibration_accumulators();
                                        }
                                        data_manager.update_calibration_accumulators(accel_data, gyro_data);
                                        

                                        let calibration_complete = {
                                            const MIN_SAMPLES: usize = 100;
                                            const MAX_SAMPLES: usize = 500;
                                            const STABILITY_WINDOW: usize = 50;
                                            
                                            if sample_count < MIN_SAMPLES {
                                                false
                                            } else if sample_count >= MAX_SAMPLES {
                                                true 
                                            } else if sample_count >= MIN_SAMPLES + STABILITY_WINDOW {
                                                data_manager.is_calibration_stable(STABILITY_WINDOW)
                                            } else {
                                                false
                                            }
                                        };
                                        
                                        if calibration_complete {
                                            data_manager.finalize_calibration();
                                            info!("Calibration complete with {} samples", sample_count);
                                        
                                            match imu.finish_calibration() {
                                                Ok(collecting_imu) => {
                                                    *imu_wrapper = ImuWrapper::Idling(collecting_imu);
                                                    state_machine.transition_to(sm::States::Idle);
                                                }
                                                Err(fault_imu) => {
                                                    *imu_wrapper = ImuWrapper::Entered_Fault(fault_imu);
                                                    state_machine.transition_to(sm::States::Fault);
                                                }
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        info!("Failed to read gyro data during calibration");
                                        state_machine.transition_to(sm::States::Fault);
                                    }
                                }
                            }
                            Err(_) => {
                                info!("Failed to read accel data during calibration");
                                state_machine.transition_to(sm::States::Fault);
                            }
                        }
                    }
                    _ => {
                        info!("Invalid state for calibration");
                        state_machine.transition_to(sm::States::Fault);
                    }
                }
            });
            Ok(())
        });
    }

    #[task(priority = 3, shared = [imu_wrapper, state_machine, adc_manager, data_manager, em, rtc])]
    async fn sm_collect(cx: sm_collect::Context) {

         match FEATURE{
            Feature::Temperature => {
                info!("Collecting data - Temperature board ready");
            }
            Feature::Pressure => {
                info!("Collecting data - Pressure board ready");
            }
            Feature::Strain => {
                info!("Collecting data - Strain board ready");
            }

        cx.shared.em.run(|| {
            (cx.shared.imu_wrapper, cx.shared.adc_manager, cx.shared.data_manager, cx.shared.state_machine, cx.shared.rtc).lock(|imu_wrapper, adc_manager, data_manager, state_machine, rtc| {
                match imu_wrapper {
                    ImuWrapper::Collecting(imu) => {
                        
                        match (adc_manager.read_adc1_data(), adc_manager.read_adc2_data()) {
                            (Ok(accel_data), Ok(gyro_data)) => {
                                info!("Collecting data - accel: {:?}, gyro: {:?}", accel_data, gyro_data);
                                
                               
                                
                                
                                
                                let timestamp = messages::FormattedNaiveDateTime(rtc.date_time().unwrap());
                                
                                
                               
                               
                                
                                if should_stop_collection {
                                    match imu.done_collecting() {
                                        Ok(idle_imu) => {
                                            *imu_wrapper = ImuWrapper::Idling(idle_imu);
                                            state_machine.transition_to(sm::States::Idle);
                                        }
                                        Err(fault_imu) => {
                                            *imu_wrapper = ImuWrapper::Entered_Fault(fault_imu);
                                            state_machine.transition_to(sm::States::Fault);
                                        }
                                    }
                                }
                            }
                            _ => {
                                info!("Failed to read sensor data during collection");
                                state_machine.transition_to(sm::States::Fault);
                            }
                        }
                    }
                    _ => {
                        info!("Invalid state for data collection");
                        state_machine.transition_to(sm::States::Fault);
                    }
                }
            });
            Ok(())
        });
        
        
        Mono::delay(100.millis()).await;
    }
}

    #[task(priority = 3, shared = [imu_wrapper, state_machine, em, data_manager])]
    async fn sm_fault(cx: sm_fault::Context) {

      info!("System in fault state - attempting recovery");

      match FEATURE{
            Feature::Temperature => {
                info!("Fault state - Temperature board ready");
            }
            Feature::Pressure => {
                info!("Fault state - Pressure board ready");
            }
            Feature::Strain => {
                info!("Fault state - Strain board ready");
            }
        }
        
        cx.shared.em.run(|| {
            (cx.shared.imu_wrapper, cx.shared.state_machine, cx.shared.data_manager).lock(|imu_wrapper, state_machine, data_manager| {
                
                info!("Fault state entered, logging error condition");
                
                match imu_wrapper {
                    ImuWrapper::Entered_Fault(imu) => {
                       
                        info!("Attempting fault recovery");
                        
                        
                        let recovery_successful = {
                            let mut recovery_steps = Vec::new();
                            
                            
                            let reset_ok = match imu.reset() {
                                Ok(_) => {
                                    info!("IMU reset successful");
                                    true
                                },
                                Err(e) => {
                                    error!("IMU reset failed: {:?}", e);
                                    false
                                }
                            };
                            recovery_steps.push(("Reset", reset_ok));
                            
                            
                            let init_ok = if reset_ok {
                                match imu.initialize() {
                                    Ok(_) => {
                                        info!("IMU re-initialization successful");
                                        true
                                    },
                                    Err(e) => {
                                        error!("IMU initialization failed: {:?}", e);
                                        false
                                    }
                                }
                            } else { false };
                            recovery_steps.push(("Initialize", init_ok));
                            
                            
                            let comm_ok = if init_ok {
                                match imu.read_device_id() {
                                    Ok(id) if id == EXPECTED_DEVICE_ID => {
                                        info!("IMU communication verified");
                                        true
                                    },
                                    Ok(wrong_id) => {
                                        error!("Wrong device ID: expected {}, got {}", EXPECTED_DEVICE_ID, wrong_id);
                                        false
                                    },
                                    Err(e) => {
                                        error!("Communication test failed: {:?}", e);
                                        false
                                    }
                                }
                            } else { false };
                            recovery_steps.push(("Communication", comm_ok));
                            
                            
                            let sensor_ok = if comm_ok {
                                match imu.read_acceleration() {
                                    Ok(accel) => {
                                        
                                        let magnitude = accel.magnitude();
                                        let readings_valid = magnitude > 0.1 && magnitude < 50.0; // Adjust threshold *****
                                        if readings_valid {
                                            info!("Sensor readings validated");
                                        } else {
                                            warn!("Sensor readings out of expected range: {}", magnitude);
                                        }
                                        readings_valid
                                    },
                                    Err(e) => {
                                        error!("Sensor reading test failed: {:?}", e);
                                        false
                                    }
                                }
                            } else { false };
                            recovery_steps.push(("Sensor Test", sensor_ok));
                            
                           
                            for (step, success) in recovery_steps.iter() {
                                info!("Recovery step '{}': {}", step, if *success { "PASS" } else { "FAIL" });
                            }
                            
                            
                            reset_ok && init_ok && comm_ok && sensor_ok
                        };
                    
                        
                        if recovery_successful {
                            info!("Fault recovery successful");
                            let recovered_imu = imu.exit_fault();
                            *imu_wrapper = ImuWrapper::Idling(recovered_imu);
                            state_machine.transition_to(sm::States::Idle);
                        } else {
                            info!("Fault recovery failed, remaining in fault state");
                            
                        }
                    }
                    _ => {
                        info!("Unexpected state in fault handler");
                    }
                }
            });
            Ok(())
        });
        
        Mono::delay(5.secs()).await;
    }

    #[task(priority = 3, shared = [imu_wrapper, state_machine, em])]
    async fn sm_idle(cx: sm_idle::Context) {
        match FEATURE{
            Feature::Temperature => {
                info!("System in idle state - Temperature board ready");

                let temp_sensor_pin = gpioa.pa1.into_analog(); //find actual pin
                let mut interval = interval(Duration::from_secs(100)); //adjust appropiately millisecond delay
                loop{
                    interval.tick().await;
                    let data_t = self.adc.read_adc1_data(NegativeInpMux::AIN1, PositiveInpMux::AIN0);
            
                    if let Ok(bytes) = data_t {
                        //convert data to signed i32
                        let raw_t = i32::from_be_bytes(bytes);

                        let adc_max_t = (1<<23) as f32;
                        let v_ref_t = 3.3; //need to find actual adc voltage reference from MCU
                        let voltage_t = (raw_t as f32 / adc_max_t) * v_ref_t;

                        let temperature_celsius = (voltage_t - 0.5) * 100.0;

                        }

                        
                        if temperature_celsius > 50.0 {
                            info!("Temperature reading too high: {} °C", temperature_celsius);
                            state_machine.transition_to(sm::States::Fault);
                        } else if temperature_celsius < 10.0{ 
                            info!("Temperature reading too low: {} °C", temperature_celsius);
                            state_machine.transition_to(sm::States::Fault);
                        }    else {
                            info!("Temperature reading: {} °C", temperature_celsius);
                        }


            }
        }
            Feature::Pressure => {
                info!("System in idle state - Pressure board ready");

                let pressure_pin = gpioa.pa2.into_analog(); //find actual pin
                let mut interval = interval(Duration::from_secs(100)); //adjust appropiately millisecond delay
                loop{
                    interval.tick().await;
                    let data_p = self.adc.read_adc1_data(NegativeInpMux::AIN1, PositiveInpMux::AIN0);
            
                    if let Ok(bytes) = data_p {

                        let adc_max_p = (1<<23) as f32;
                        let v_ref_p = 3.3; //need to find actual adc voltage reference from MCU
                        let raw_p = i32::from_be_bytes(bytes);
                        let voltage_p = (raw_p as f32 / adc_max_p) * v_ref_p;

                        let max_pressure = 101.3;


                        let v_min = 0;  //find actual values
                        let v_max = 4.5;
                        let p_min = 26.5;
                        let p_max = max_pressure;

                        let slope = (p_max - p_min) / (v_max - v_min);
                        let offset = p_min - slope * v_min;


                        let pressure_kpa = slope * voltage_p + offset;

                       
                    }

                        
                        if pressure_kpa > 150.0 { //verify this threshold
                            info!("Pressure reading too high: {} kPa", pressure_kpa);
                            state_machine.transition_to(sm::States::Fault);
                        } else if pressure_kpa < 20.0 {
                            info!("Pressure reading too low: {} kPa", pressure_kpa);
                            state_machine.transition_to(sm::States::Fault);
                        } else {
                            info!("Pressure reading: {} kPa", pressure_kpa);
                        }
            }
            Feature::Strain => {
                info!("System in idle state - Strain board ready");
                let strain_pin = gpioa.pa3.into_analog(); //find actual pin
                let mut interval = interval(Duration::from_secs(100)); //adjust appropiately millisecond delay
                loop{
                    interval.tick().await;
                    let data_s = self.adc.read_adc1_data(NegativeInpMux::AIN1, PositiveInpMux::AIN0);
            
                    if let Ok(bytes) = data_s {

                        let adc_max_s = (1<<23) as f32;
                        let v_ref_s = 3.3; //need to find actual adc voltage reference from MCU
                        let raw_s = i32::from_be_bytes(bytes);
                        let voltage_s = (raw_s as f32 / adc_max_s) * v_ref_s;
                        let gauge_factor = 2.0; //example value, find actual gauge factor
                        let e_voltage = 1.0; //example value, find actual excitation voltage
                        let strain_value = (voltage_s / (gauge_factor * e_voltage));

                        
                    }

                        
                        if strain_value > 2000.0 { //verify this threshold
                            info!("Strain reading too high: {} microstrain", strain_value);
                            state_machine.transition_to(sm::States::Fault);
                        } else if strain_value < -2000.0 {
                            info!("Strain reading too low: {} microstrain", strain_value);
                            state_machine.transition_to(sm::States::Fault);
                        } else {
                            info!("Strain reading: {} microstrain", strain_value);
                        }
            }
        }

        cx.shared.em.run(|| {
            (cx.shared.imu_wrapper, cx.shared.state_machine).lock(|imu_wrapper, state_machine| {
                match imu_wrapper {
                    ImuWrapper::Idling(imu) => {
                        info!("System in idle state");
                        
                        
                        
                        let needs_calibration = {
                            
                            let current_bias = imu.get_bias_estimate();
                            current_bias.magnitude() > CALIBRATION_THRESHOLD
                        };
                        
                        
                        let start_collection = {
                            let accel_data = imu.read_acceleration();
                            accel_data.magnitude() > MOTION_THRESHOLD
                        };
                        
                        if needs_calibration {
                            match imu.to_calibration() {
                                Ok(calibrating_imu) => {
                                    *imu_wrapper = ImuWrapper::Calibrating(calibrating_imu);
                                    state_machine.transition_to(sm::States::Calibration);
                                    info!("Transitioning to calibration state");
                                }
                                Err(fault_imu) => {
                                    *imu_wrapper = ImuWrapper::Entered_Fault(fault_imu);
                                    state_machine.transition_to(sm::States::Fault);
                                }
                            }
                        } else if start_collection {
                            match imu.to_collection() {
                                Ok(collecting_imu) => {
                                    *imu_wrapper = ImuWrapper::Collecting(collecting_imu);
                                    state_machine.transition_to(sm::States::Collection);
                                    info!("Transitioning to collection state");
                                }
                                Err(fault_imu) => {
                                    *imu_wrapper = ImuWrapper::Entered_Fault(fault_imu);
                                    state_machine.transition_to(sm::States::Fault);
                                }
                            }
                        }
                        
                    
                    }
                    _ => {
                        info!("Invalid state for idle handler");
                        state_machine.transition_to(sm::States::Fault);
                    }
                }
            });
            Ok(())
        });
        
        
        Mono::delay(1.secs()).await;
    }

    #[task(priority = 3, shared = [imu_wrapper, state_machine, adc_manager, em])]
    async fn sm_init(cx: sm_init::Context) {
        info!("Initializing system");

       match FEATURE{
        Feature::Temperature => {
            info!("Initializing Temperature board")

            let temp_sensor_pin = gpioa.pa1.into_analog(); //find actual pin
            //get data from adc for temperature
            let data_t = self.adc.read_adc1_data(NegativeInpMux::AIN1, PositiveInpMux::AIN0);
            
            if let Ok(bytes) = data_t {
                //convert data to signed i32
                let raw_t = i32::from_be_bytes(bytes);

                let adc_max_t = (1<<23) as f32;
                let v_ref_t = 3.3; //need to find actual adc voltage reference from MCU
                let voltage_t = (raw_t as f32 / adc_max_t) * v_ref_t;

                let temperature_celsius = (voltage_t - 0.5) * 100.0;

                info!("temperature reading around: {} °C", temperature_celsius);


            }



           
            //*****ask about where to store temperature data**************

            
        }
        Feature::Pressure  => {
            info!("Initializing Pressure Board");

            let pressure_pin = gpioa.pa2.into_analog(); //find actual pin

            let data_p = self.adc.read_adc2_data(NegativeInpMux::AIN1, PositiveInpMux::AIN0);

            if let Ok(bytes) = data_p {

                let adc_max_p = (1<<23) as f32;
                let v_ref_p = 3.3; //need to find actual adc voltage reference from MCU
                let raw_p = i32::from_be_bytes(bytes);
                let voltage_p = (raw_p as f32 / adc_max_p) * v_ref_p;

                let max_pressure = 101.3;


                let v_min = 0;  //find actual values
                let v_max = 4.5;
                let p_min = 26.5;
                let p_max = max_pressure;

                let slope = (p_max - p_min) / (v_max - v_min);
                let offset = p_min - slope * v_min;


                let pressure_kpa = slope * voltage_p + offset;  //verify formula

                info!("pressure reading around: {} kpa", pressure_kpa);
            }
        }
        Feature::Strain => {
            let strain_pin = gpioa.pa3.into_analog(); //find actual pin

            let data_s = self.adc.read_adc3_data(NegativeInpMux::AIN1, PositiveInpMux::AIN0);

            if let Ok(bytes) = data_p {

                let adc_max_s = (1<<23) as f32;
                let v_ref_s = 3.3; //need to find actual adc voltage reference from MCU
                let raw_s = i32::from_be_bytes(bytes);
                let voltage_s = (raw_s as f32 / adc_max_s) * v_ref_s;
                let gauge_factor = 2.0; //example value, find actual gauge factor
                let e_voltage = 1.0; //example value, find actual excitation voltage
                let strain_value = (voltage_s / (gauge_factor * e_voltage));

                info!("strain reading around: {} microstrain", strain_value);
            }

        }
       }
        
        cx.shared.em.run(|| {
            (cx.shared.adc_manager, cx.shared.imu_wrapper, cx.shared.state_machine).lock(|adc_manager, imu_wrapper, state_machine| {
                
                match adc_manager.init_adc1() { 
                    Ok(_) => info!("ADC1 initialized successfully"),
                    Err(_) => {
                        info!("Failed to initialize ADC1");
                        state_machine.transition_to(sm::States::Fault);
                        return Ok(());
                    }
                }
                
                
                match (adc_manager.read_adc1_data(), adc_manager.read_adc2_data()) { 
                        info!("Initial sensor readings successful - accel: {:?}, gyro: {:?}", accel_data, gyro_data);
                        
                        
                        match imu_wrapper { 
                            ImuWrapper::Uninitialized(uninitialized_imu) => {
                                let idle_imu = uninitialized_imu.initial_state();
                                *imu_wrapper = ImuWrapper::Idling(idle_imu);
                                state_machine.transition_to(sm::States::Idle);
                                info!("System initialization complete, transitioning to idle");
                            }
                            _ => {
                                info!("IMU wrapper in unexpected state during init");
                                state_machine.transition_to(sm::States::Fault);
                            }
                        }
                    }
                    _ => {
                        info!("Failed initial sensor readings");
                        state_machine.transition_to(sm::States::Fault);
                    }
                }
                Ok(())
            })
        });
    }

    #[task(priority = 3, binds = EXTI15_10, shared = [adc_manager], local = [adc1_int])]
    fn adc1_data_ready(mut cx: adc1_data_ready::Context) {
        info!("new data available come through");
        cx.shared.adc_manager.lock(|adc_manager| {
            let data = adc_manager.read_adc1_data();
            match data {
                Ok(data) => {
                    info!("data: {:?}", data);
                }
                Err(_) => {
                    info!("Error reading data");
                }
            }
            // change the inpmux
            adc_manager.set_adc1_inpmux(
                ads126x::register::NegativeInpMux::AIN1,
                ads126x::register::PositiveInpMux::AIN0,
            );
        });
        cx.local.adc1_int.clear_interrupt_pending_bit();
    }

    #[task(priority = 3, binds = EXTI3, shared = [adc_manager], local = [adc2_int])]
    fn adc2_data_ready(mut cx: adc2_data_ready::Context) {
        info!("new data available come through");
        cx.shared.adc_manager.lock(|adc_manager| {
            let data = adc_manager.read_adc2_data();
            match data {
                Ok(data) => {
                    info!("data: {:?}", data);
                }
                Err(_) => {
                    info!("Error reading data");
                }
            }
            adc_manager.set_adc2_inpmux(
                ads126x::register::NegativeInpMux::AIN1,
                ads126x::register::PositiveInpMux::AIN0,
            );
        });
        cx.local.adc2_int.clear_interrupt_pending_bit();
    }

    #[task(priority = 3, shared = [data_manager, &em, rtc])]
    async fn reset_reason_send(mut cx: reset_reason_send::Context) {
        let reason = cx
            .shared
            .data_manager
            .lock(|data_manager| data_manager.reset_reason.take());
        match reason {
            Some(reason) => {
                let message = messages::CanMessage::new(
                    cx.shared
                        .rtc
                        .lock(|rtc| messages::FormattedNaiveDateTime(rtc.date_time().unwrap())),
                    COM_ID,
                    CanData::Common(reason.into()),
                );

                cx.shared.em.run(|| {
                    spawn!(queue_data_internal, message)?;
                    Ok(())
                });
            }
            None => return,
        }
    }

    #[task(priority = 3)]
    async fn delay(_cx: delay::Context, delay: u32) {
        Mono::delay(delay.millis()).await;
    }

    #[task(shared = [data_manager, &em, rtc])]
    async fn state_send(mut cx: state_send::Context) {
        let state_data = cx
            .shared
            .data_manager
            .lock(|data_manager| data_manager.state.clone());
        cx.shared.em.run(|| {
            if let Some(x) = state_data {
                let can_data: CanData = CanData::Common(x.into());
                let message = CanMessage::new(
                    cx.shared
                        .rtc
                        .lock(|rtc| messages::FormattedNaiveDateTime(rtc.date_time().unwrap())),
                    COM_ID,
                    can_data,
                );
                cx.shared.em.run(|| {
                    spawn!(queue_data_internal, message)?;
                    Ok(())
                });
            } // if there is none we still return since we simply don't have data yet.
            Ok(())
        });
        Mono::delay(5.secs()).await;
        // spawn_after!(state_send, ExtU64::secs(5)).ok();
    }

    /**
     * Sends information about the sensors.
     */
    #[task(priority = 3, shared = [data_manager, rtc, &em])]
    async fn sensor_send(mut cx: sensor_send::Context) {
        let sensors = cx
            .shared
            .data_manager
            .lock(|data_manager| data_manager.temperature.take());

        cx.shared.em.run(|| {
            match sensors {
                Some(x) => {
                    for sensor in x.iter() {
                        let message = CanMessage::new(
                            messages::FormattedNaiveDateTime(
                                cx.shared.rtc.lock(|rtc| rtc.date_time().unwrap()),
                            ),
                            COM_ID,
                            CanData::Temperature(*sensor),
                        );
                        spawn!(queue_data_internal, message)?;
                    }
                }
                None => {
                    info!("No sensor data to send");
                }
            }
            Ok(())
        });
    }

    /// Callback for our logging library to access the needed resources.
    pub fn queue_log_message(d: impl Into<CanData>) {
        send_log_intermediate::spawn(d.into()).ok();
    }

    #[task(priority = 3, local = [can_sender], shared = [&em])]
    async fn queue_data_internal(cx: queue_data_internal::Context, m: CanMessage) {
        match cx.local.can_sender.send(m).await {
            // Preferably clean this up to be handled by the error manager.
            Ok(_) => {}
            Err(_) => {
                info!("Failed to send data");
            }
        }
    }

    #[task(priority = 3, shared = [rtc, &em])]
    async fn send_log_intermediate(mut cx: send_log_intermediate::Context, m: CanData) {
        cx.shared.em.run(|| {
            cx.shared.rtc.lock(|rtc| {
                let message = messages::CanMessage::new(
                    messages::FormattedNaiveDateTime(rtc.date_time().unwrap()),
                    COM_ID,
                    m,
                );

                spawn!(queue_data_internal, message)?;
                Ok(())
            })
        });
    }

    // #[task(priority = 2, binds = FDCAN1_IT0, shared = [can_command_manager, data_manager, &em])]
    // fn can_command(mut cx: can_command::Context) {
    //     // info!("CAN Command");
    //     cx.shared.can_command_manager.lock(|can| {
    //         cx.shared
    //             .data_manager
    //             .lock(|data_manager| cx.shared.em.run(|| can.process_data(data_manager)));
    //     })
    // }

    // #[task( priority = 3, binds = FDCAN2_IT0, shared = [&em, can_data_manager, data_manager])]
    // fn can_data(mut cx: can_data::Context) {
    //     cx.shared.can_data_manager.lock(|can| {
    //         {
    //             cx.shared.data_manager.lock(|data_manager| {
    //                 cx.shared.em.run(|| {
    //                     can.process_data(data_manager)?;
    //                     Ok(())
    //                 })
    //             })
    //         }
    //     });
    // }

    // #[task(priority = 2, shared = [&em, can_data_manager, data_manager])]
    // async fn send_data_internal(
    //     mut cx: send_data_internal::Context,
    //     mut receiver: Receiver<'static, CanMessage, DATA_CHANNEL_CAPACITY>,
    // ) {
    //     loop {
    //         if let Ok(m) = receiver.recv().await {
    //             cx.shared.can_data_manager.lock(|can| {
    //                 cx.shared.em.run(|| {
    //                     can.send_message(m)?;
    //                     Ok(())
    //                 })
    //             });
    //         }
    //     }
    // }

    // #[task(priority = 2, shared = [&em, can_command_manager, data_manager])]
    // async fn send_command_internal(mut cx: send_command_internal::Context, m: CanMessage) {
    //     cx.shared.can_command_manager.lock(|can| {
    //         cx.shared.em.run(|| {
    //             can.send_message(m)?;
    //             Ok(())
    //         })
    //     });
    // }

    #[task(priority = 1, local = [led_red, led_green], shared = [&em])]
    async fn blink(cx: blink::Context) {
        loop {
            if cx.shared.em.has_error() {
                cx.local.led_red.toggle();
                Mono::delay(500.millis()).await;
            } else {
                cx.local.led_green.toggle();
                Mono::delay(2000.millis()).await;
            }
        }
    }

    #[task(priority = 3, shared = [&em])]
    async fn sleep_system(_cx: sleep_system::Context) {
        // in here we can stop the ADCs.
    }
}

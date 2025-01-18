use crate::state_machine::{RocketStates, State, StateMachineContext, TransitionInto};
use crate::types::COM_ID;
use crate::{no_transition, transition};
use defmt::{write, Format, Formatter};
use messages::command::{Command, RadioRate, RadioRateChange};
use rtic::mutex::Mutex;

#[derive(Debug, Clone)]
pub struct Init {}

impl State for Init {
    fn enter(&self, context: &mut StateMachineContext) {
        let radio_rate_change = RadioRateChange {
            rate: RadioRate::Fast,
        };
        let message_com = Message::new(
            cortex_m::interrupt::free(|cs| {
                let mut rc = RTC.borrow(cs).borrow_mut();
                let rtc = rc.as_mut().unwrap();
                rtc.count32()
            }),
            COM_ID,
            Command::new(radio_rate_change),
        );
        context.shared_resources.shared.data_manager.lock(|data| {

        }); 
        context.shared_resources.can0.lock(|can| {
            context.shared_resources.em.run(|| {
                can.send_message(message_com)?;
                Ok(())
            })
        });
    }
    fn step(&mut self, context: &mut StateMachineContext) -> Option<RocketStates> {
        context.shared_resources.data_manager.lock(|data| {
            if data.is_falling() {
                transition!(self, Descent)
            } else {
                no_transition!()
            }
        })
    }
}

impl TransitionInto<Init> for WaitForTakeoff {
    fn transition(&self) -> Init {
        Init {}
    }
}

impl Format for Init {
    fn format(&self, f: Formatter) {
        write!(f, "Init")
    }
}

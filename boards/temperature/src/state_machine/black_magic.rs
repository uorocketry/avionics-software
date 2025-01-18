use crate::state_machine::RocketEvents;
use core::fmt::Debug;
use defmt::{info, Format};
use enum_dispatch::enum_dispatch;

use super::TemperatureBoardState;

/// Trait that all states implement. Ignore this, not super important
#[enum_dispatch]
pub trait State: Debug {
    fn enter(&self, _context: &mut crate::app::__rtic_internal_run_sm_Context)
    where
        Self: Format,
    {
        info!("Enter {:?}", self)
    }
    fn exit(&self)
    where
        Self: Format,
    {
        info!("Exit {:?}", self)
    }
    fn event(&mut self, event: RocketEvents) -> Option<TemperatureBoardState>;
    fn step(&mut self, context: &mut crate::app::__rtic_internal_run_sm_Context) -> Option<TemperatureBoardState>;
}

/// Transition Trait
pub trait TransitionInto<T> {
    fn transition(&self) -> T;
}

#[macro_export]
macro_rules! transition {
    ($self:ident, $i:ident) => {
        Some(TransitionInto::<$i>::transition($self).into())
    };
}

#[macro_export]
macro_rules! no_transition {
    () => {
        None
    };
}

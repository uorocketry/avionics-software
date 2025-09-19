use argus::state_machine::config::MAX_CONCURRENT_TASKS;
use defmt::Format;
use embassy_sync::{
	blocking_mutex::raw::CriticalSectionRawMutex,
	watch::{Receiver, Watch},
};
use smlang::statemachine;

// We're only using smlang's state transition locking feature, not their action/guard
statemachine! {
	derive_states: [Clone,Debug, Format],
	derive_events: [Clone, Debug, Format],
	transitions: {
		*Idle + StartRecordingRequested = Recording,
		Idle + CalibrationRequested = Calibrating,
		Recording + CalibrationRequested = Calibrating,
		Calibrating + FinishCalibration = Idle,
		Recording + StopRecordingRequested = Idle,
	}
}

pub struct Context;
impl StateMachineContext for Context {}

pub type StateWatch = Watch<CriticalSectionRawMutex, States, MAX_CONCURRENT_TASKS>;
pub type StateReceiver = Receiver<'static, CriticalSectionRawMutex, States, MAX_CONCURRENT_TASKS>;

use core::future::Future;

use defmt::{error, info};
use embassy_futures::select::{select, Either};

use crate::{
	state_machine::types::{Context, Events, StateMachine, StateReceiver, StateWatch, States},
	utils::types::AsyncMutex,
};

// Current state cannot be stored on the state machine itself, as we need to be able to
// reference it from multiple async tasks. So we store it in a static watch globally instead.
static CURRENT_STATE: StateWatch = StateWatch::new();

// Encapsulates the logic for managing the state machine
pub struct StateMachineOrchestrator {
	state_machine: StateMachine<Context>,
}
impl Default for StateMachineOrchestrator {
	fn default() -> Self {
		Self::new()
	}
}

impl StateMachineOrchestrator {
	pub fn new() -> Self {
		Self {
			state_machine: StateMachine::new(Context {}),
		}
	}

	pub fn dispatch_event(
		&mut self,
		event: Events,
	) {
		let previous_state = self.state_machine.state().clone();
		let result = self.state_machine.process_event(event.clone());
		match result {
			Ok(state) => {
				CURRENT_STATE.sender().send(state.clone());
				info!("State changed from {:?} to {:?} due to event {:?}", previous_state, state, event);
			}
			Err(_) => {
				error!("Invalid event dispatched in state {:?}: {:?}", previous_state, event);
			}
		}
	}
}

// Encapsulates the logic needed by a worker task
pub struct StateMachineWorker {
	current_state: StateReceiver,
	orchestrator: &'static AsyncMutex<StateMachineOrchestrator>,
}
impl StateMachineWorker {
	pub fn new(orchestrator: &'static AsyncMutex<StateMachineOrchestrator>) -> Self {
		Self {
			current_state: CURRENT_STATE.receiver().unwrap(),
			orchestrator,
		}
	}

	pub async fn run_once<Err, Act, Fut>(
		&mut self,
		desired_states: &[States],
		mut action: Act,
	) -> Result<(), Err>
	where
		Act: FnMut(&'static AsyncMutex<StateMachineOrchestrator>) -> Fut,
		Fut: Future<Output = Result<(), Err>>, {
		self.current_state.changed_and(|state| desired_states.contains(state)).await;
		action(self.orchestrator).await
	}

	pub async fn run_while<Err, Act, Fut>(
		&mut self,
		desired_states: &[States],
		mut action: Act,
	) -> Result<(), Err>
	where
		Act: FnMut(&'static AsyncMutex<StateMachineOrchestrator>) -> Fut,
		Fut: Future<Output = Result<(), Err>>, {
		if desired_states.is_empty() {
			return Ok(());
		}

		// Runs indefinitely
		loop {
			// Wait until we're in one of the desired states
			if !desired_states.contains(&self.current_state.get().await) {
				self.current_state.changed_and(|state| desired_states.contains(state)).await;
			}

			// We're now in a desired state, run the action until the state changes
			loop {
				// Race between state change or action completion
				let state_changed = self.current_state.changed();
				let action_finished = action(self.orchestrator);

				match select(state_changed, action_finished).await {
					Either::First(_) => {
						break;
					}
					Either::Second(_) => {
						let current_state = self.current_state.get().await;
						if !desired_states.contains(&current_state) {
							// debug!("State changed while action was running, stopping action.");
							break;
						}
					}
				}
			}
		}
	}
}

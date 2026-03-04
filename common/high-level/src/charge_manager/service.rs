// !! NOTE VELOCITY IS RELATIVE TO 0 (EARTH), NOT TO ROCKET !!

use uor_drivers::ejection_channel::driver::EjectionChannel;
use uor_utils::{
	buffer_types::rase::{altitude::AltitudeData, pose::PoseData},
	utils::data_structures::pipe::Pipe,
};
#[derive(PartialEq)]
pub enum RocketState {
	InFlight,
	Armed,
	DrogueDeployed,
	MainDeployed,
}

pub struct ChargeManager<'a, const NUM_MAIN: usize, const NUM_DROGUE: usize> {
	pose_pipe: &'a Pipe<PoseData>,
	altitude_pipe: &'a Pipe<AltitudeData>,
	main_channels: [EjectionChannel<'a>; NUM_MAIN],
	drogue_channels: [EjectionChannel<'a>; NUM_DROGUE],
	altitude_degraded: bool,
	current_state: RocketState,
	reset_detected_flag: bool,
}

impl<'a, const NUM_MAIN: usize, const NUM_DROGUE: usize> ChargeManager<'a, NUM_MAIN, NUM_DROGUE> {
	pub fn new(
		pose_pipe: &'a Pipe<PoseData>,
		altitude_pipe: &'a Pipe<AltitudeData>,
		main_channels: [EjectionChannel<'a>; NUM_MAIN],
		drogue_channels: [EjectionChannel<'a>; NUM_DROGUE],
		altitude_degraded: bool,
		reset_detected_flag: bool,
	) -> ChargeManager<'a, NUM_MAIN, NUM_DROGUE> {
		ChargeManager {
			pose_pipe: pose_pipe,
			altitude_pipe: altitude_pipe,
			main_channels: main_channels,
			drogue_channels: drogue_channels,
			altitude_degraded: false,
			current_state: RocketState::InFlight,
			reset_detected_flag: reset_detected_flag,
		}
	}

	pub fn check_drogue_continuity(&mut self) -> bool {
		let mut continuity = true;
		for i in &mut self.drogue_channels {
			if !i.check_continuity() {
				continuity = false;
				break;
			}
		}
		continuity
	}

	pub fn update(&mut self) {
		let altitude_data = self.altitude_pipe.read();
		let pose_data = self.pose_pipe.read();

		// Checking for software arm conditions (Kinda dangerous to force arm on watchdog resets but a better solution is yet to be found)
		if altitude_data.altitude.feet() > 20000 || self.reset_detected_flag {
			// Arm drogue channels
			match self.current_state {
				RocketState::InFlight => {
					for i in &mut self.drogue_channels {
						i.arm();
					}
					// Arm main channels
					for i in &mut self.main_channels {
						i.arm();
					}
					self.current_state = RocketState::Armed;
					self.reset_detected_flag = false;
				}
				// Check for apogee (vertical velocity=0 +- 3m/s)
				RocketState::Armed => {
					if pose_data.velocity.k.fmeters().abs() < 3.0 {
						for i in &mut self.drogue_channels {
							i.deploy_charge();
						}
						self.current_state = RocketState::DrogueDeployed;
					}
				}
				_ => (),
			}

		// TODO: Check for main
		} else if self.current_state == RocketState::DrogueDeployed && altitude_data.altitude.ffeet() < 1500.0 {
			for i in &mut self.main_channels {
				i.deploy_charge();
			}
			self.current_state = RocketState::MainDeployed;
		}
	}
}

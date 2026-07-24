#[derive(PartialEq, Clone)]

pub enum RocketState {
	InFlight,
	Armed,
	DrogueDeployed,
	MainDeployed,
}

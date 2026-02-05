use defmt::Format;

#[derive(PartialEq, Clone, Format)]
pub enum EjectionChannelStates {
	NoContinuity,
	ContinuityLost,
	Idle,
	Armed,
	Deployed,
	ConfirmedDeployed,
}

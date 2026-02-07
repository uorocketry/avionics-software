use defmt::Format;

#[derive(PartialEq, Clone, Format)]
pub enum EjectionChannelStates {
	Unknown,
	NoContinuity,
	ContinuityLost,
	Armed,
	Deployed,
	ConfirmedDeployed,
}

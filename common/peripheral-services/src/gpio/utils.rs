use embassy_stm32::gpio::{Pull, Speed};

pub enum GPIOMode {
	Input(Pull),
	Output(Speed),
	InputOutput(Speed),
	InputOutputPull(Speed, Pull),
}

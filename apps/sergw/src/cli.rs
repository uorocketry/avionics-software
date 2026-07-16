use std::net::SocketAddr;

use clap::{Parser, Subcommand, ValueEnum};
use serialport::{DataBits, Parity, StopBits};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
	#[command(subcommand)]
	pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
	/// List available serial ports
	Ports {
		/// Include non-USB ports as well
		#[arg(long)]
		all: bool,
		/// Show detailed metadata
		#[arg(long)]
		verbose: bool,
		/// Output format
		#[arg(long, value_enum, default_value_t = PortsFormat::Text)]
		format: PortsFormat,
	},
	/// Bridge a serial port to TCP
	Listen(Listen),

	#[cfg(target_os = "linux")]
	/// Mock utilities
	Mock {
		#[command(subcommand)]
		cmd: MockCmd,
	},
}

#[derive(Parser, Clone, Debug)]
pub struct Listen {
	/// Serial port to open (auto-select if exactly one is found and this is omitted)
	#[arg(long)]
	pub serial: Option<String>,

	/// Baud rate
	#[arg(long, default_value_t = 115_200)]
	pub baud: u32,

	/// TCP listen address
	#[arg(long, default_value = "127.0.0.1:5656")]
	pub host: SocketAddr,

	/// Data bits
	#[arg(long, value_enum, default_value_t = DataBitsOpt::Eight)]
	pub data_bits: DataBitsOpt,

	/// Parity
	#[arg(long, value_enum, default_value_t = ParityOpt::None)]
	pub parity: ParityOpt,

	/// Stop bits
	#[arg(long, value_enum, default_value_t = StopBitsOpt::One)]
	pub stop_bits: StopBitsOpt,

	/// Buffer capacity (messages) for internal channels
	#[arg(long, default_value_t = 4096)]
	pub buffer: usize,

	/// Run without the TUI (daemon mode, logs output to stderr)
	#[arg(long)]
	pub no_tui: bool,
}

#[cfg(target_os = "linux")]
#[derive(Subcommand, Clone, Debug)]
pub enum MockCmd {
	/// Create a PTY-backed serial device and open a chat UI bound to it
	Serial {
		/// Optionally create a symlink to the slave PTY at this path (cannot force /dev/pts/N)
		#[arg(long)]
		alias: Option<String>,
	},
	/// Open a chat UI connected to a TCP server (replaces `socat - TCP:host:port`)
	Listener {
		#[command(flatten)]
		chat: Chat,
	},
}

#[derive(Parser, Clone, Debug)]
pub struct Chat {
	/// TCP server to connect to (e.g. 127.0.0.1:5656)
	#[arg(long, default_value = "127.0.0.1:5656")]
	pub host: std::net::SocketAddr,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum DataBitsOpt {
	Five,
	Six,
	Seven,
	Eight,
}

impl From<DataBitsOpt> for DataBits {
	fn from(v: DataBitsOpt) -> Self {
		match v {
			DataBitsOpt::Five => DataBits::Five,
			DataBitsOpt::Six => DataBits::Six,
			DataBitsOpt::Seven => DataBits::Seven,
			DataBitsOpt::Eight => DataBits::Eight,
		}
	}
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ParityOpt {
	None,
	Odd,
	Even,
}

impl From<ParityOpt> for Parity {
	fn from(v: ParityOpt) -> Self {
		match v {
			ParityOpt::None => Parity::None,
			ParityOpt::Odd => Parity::Odd,
			ParityOpt::Even => Parity::Even,
		}
	}
}

#[derive(ValueEnum, Clone, Debug)]
pub enum StopBitsOpt {
	One,
	Two,
}

impl From<StopBitsOpt> for StopBits {
	fn from(v: StopBitsOpt) -> Self {
		match v {
			StopBitsOpt::One => StopBits::One,
			StopBitsOpt::Two => StopBits::Two,
		}
	}
}

#[derive(ValueEnum, Clone, Debug)]
pub enum PortsFormat {
	Text,
	Json,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_listen_defaults() {
		let cli = Cli::parse_from(["sergw", "listen"]);
		match cli.command.unwrap() {
			Commands::Listen(l) => {
				assert_eq!(l.serial, None);
				assert_eq!(l.baud, 115_200);
				assert_eq!(l.host, "127.0.0.1:5656".parse().unwrap());
				assert!(matches!(l.data_bits, DataBitsOpt::Eight));
				assert!(matches!(l.parity, ParityOpt::None));
				assert!(matches!(l.stop_bits, StopBitsOpt::One));
				assert_eq!(l.buffer, 4096);
				assert!(!l.no_tui);
			}
			_ => panic!("expected listen"),
		}
	}

	#[test]
	fn parse_listen_values() {
		let cli = Cli::parse_from([
			"sergw",
			"listen",
			"--serial",
			"/dev/ttyUSB9",
			"--baud",
			"57600",
			"--host",
			"0.0.0.0:9000",
			"--data-bits",
			"seven",
			"--parity",
			"even",
			"--stop-bits",
			"two",
			"--buffer",
			"123",
			"--no-tui",
		]);
		match cli.command.unwrap() {
			Commands::Listen(l) => {
				assert_eq!(l.serial.as_deref(), Some("/dev/ttyUSB9"));
				assert_eq!(l.baud, 57_600);
				assert_eq!(l.host, "0.0.0.0:9000".parse().unwrap());
				assert!(matches!(l.data_bits, DataBitsOpt::Seven));
				assert!(matches!(l.parity, ParityOpt::Even));
				assert!(matches!(l.stop_bits, StopBitsOpt::Two));
				assert_eq!(l.buffer, 123);
				assert!(l.no_tui);
			}
			_ => panic!("expected listen"),
		}
	}

	#[test]
	fn parse_ports_json() {
		let cli = Cli::parse_from(["sergw", "ports", "--format", "json"]);
		match cli.command.unwrap() {
			Commands::Ports { format, .. } => {
				assert!(matches!(format, PortsFormat::Json));
			}
			_ => panic!("expected ports"),
		}
	}
}

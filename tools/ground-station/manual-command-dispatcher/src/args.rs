use clap::Parser;
use messages_prost::common::Node;

#[derive(Parser, Debug, Clone)]
#[command(version, about = "Manual protobuf command dispatcher (TUI)", long_about = None)]
pub struct Args {
	#[arg(
		long,
		default_value = "tcpout:127.0.0.1:5656",
		help = "Gateway MAVLink connection string (e.g., tcpout:localhost:5656)"
	)]
	pub gateway_connection_string: String,

	#[arg(long, default_value_t = Node::GroundStation as i32, help = "Default command origin node as numeric enum value")]
	pub origin_node: i32,

	#[arg(long, default_value_t = Node::Phoenix as i32, help = "Default command target node as numeric enum value")]
	pub target_node: i32,
}

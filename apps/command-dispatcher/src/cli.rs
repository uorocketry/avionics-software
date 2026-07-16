use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
	#[arg(long, default_value = "http://localhost:8080")]
	pub libsql_url: String,

	#[arg(long, default_value = "")]
	pub libsql_auth_token: String,

	#[arg(
		long,
		default_value = "tcpout:127.0.0.1:5656",
		help = "Gateway MAVLink connection string (e.g., tcpout:localhost:5656, udpin:0.0.0.0:14550)"
	)]
	pub gateway_connection_string: String,

	#[arg(long, default_value_t = 5)]
	pub poll_interval_secs: u64,

	#[arg(long, default_value_t = 30, help = "Seconds between automatic Ping keepalives; set to 0 to disable")]
	pub keepalive_interval_secs: u64,
}

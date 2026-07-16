mod connection_manager;
mod heartbeat;
mod message_handler;
mod savers;

use std::time::Duration;

use clap::Parser;
use tracing::error;
use tracing::Level;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
	// database url
	#[arg(long, default_value = "http://localhost:8080")]
	libsql_url: String,

	// Renamed mavlink_connection_string to gateway_connection_string
	#[arg(
        long,
        default_value = "tcpout:127.0.0.1:5656",
        // Renamed parameter in help text
        help = "Gateway MAVLink connection string (e.g., tcpout:localhost:5656, udpin:0.0.0.0:14550)"
    )]
	gateway_connection_string: String, // Renamed field
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	tracing_subscriber::fmt().with_max_level(Level::INFO).init();

	let args = Args::parse();

	// Start heartbeat task - Pass needed args (libsql_url is needed, maybe others)
	// Cloning args here if heartbeat needs more than just libsql_url in the future
	let args_for_heartbeat = args.clone();
	tokio::spawn(async move {
		if let Err(e) = heartbeat::run_heartbeat_task(args_for_heartbeat.libsql_url).await {
			error!("Heartbeat task failed: {:?}", e);
		}
	});

	// Establish database connection (retry handled internally by libsql if needed)
	let db_connection = connection_manager::connect_to_database(args.libsql_url).await?;

	// Keep running even if the gateway (e.g., `sergw`) is absent. Retry connect and, if the
	// message loop exits, attempt to reconnect after a short delay.
	loop {
		match connection_manager::connect_to_mavlink(&args.gateway_connection_string).await {
			Ok(mavlink_connection) => {
				if let Err(e) = message_handler::handle_messages(mavlink_connection, db_connection.clone()).await {
					error!("Message handler exited: {:?}", e);
				}
			}
			Err(e) => {
				error!("Failed to connect to MAVLink: {:?}", e);
			}
		}
		tokio::time::sleep(Duration::from_secs(5)).await;
	}
}

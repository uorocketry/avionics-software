mod cli;
use cli::Args;
mod health;
use health::run_service_status_task;
mod keepalive;
use keepalive::run_keepalive_task;
mod dispatcher;
use dispatcher::run_dispatcher;
mod commands;

use clap::Parser; // For Args::parse()
use libsql::Builder;
use tracing::{error, info}; // Only error and info are used in main
							// Tokio is brought in by #[tokio::main]

pub const SERVICE_ID: &str = "command-dispatcher";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = Args::parse();
	let start_instant = std::time::Instant::now();
	tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

	info!("{} starting...", SERVICE_ID);

	info!("Attempting to connect to database: {}", args.libsql_url);
	let db = Builder::new_remote(args.libsql_url.clone(), args.libsql_auth_token.clone())
		.build()
		.await?;
	let db_connection = match db.connect() {
		Ok(connection) => {
			info!("Database connection established successfully");
			connection
		}
		Err(e) => {
			error!("Failed to get database connection: {:?}", e);
			return Err(e.into());
		}
	};
	info!("Obtained database connection handle.");

	let db_conn_for_health = db_connection.clone();
	let service_status_handle = tokio::spawn(async move {
		if let Err(e) = run_service_status_task(db_conn_for_health).await {
			error!("Service status task exited with error: {:?}", e);
		}
	});

	let db_conn_for_keepalive = db_connection.clone();
	let keepalive_interval_secs = args.keepalive_interval_secs;
	let keepalive_handle = tokio::spawn(async move {
		if let Err(e) = run_keepalive_task(db_conn_for_keepalive, keepalive_interval_secs).await {
			error!("Command keepalive task exited with error: {:?}", e);
		}
	});

	info!("Command dispatcher initialized. Starting dispatch loop...");

	if let Err(e) = run_dispatcher(db_connection, args, start_instant).await {
		error!("Dispatcher loop exited with critical error: {:?}", e);
		service_status_handle.abort();
		keepalive_handle.abort();
		return Err(e);
	}

	Ok(())
}

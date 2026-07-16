use std::time::{Duration, SystemTime};

use libsql::Builder;
use tracing::{info, warn};

pub async fn run_heartbeat_task(db_url: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let hostname = hostname::get()
		.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?
		.into_string()
		.map_err(|_| Box::from("Failed to convert OsString to String") as Box<dyn std::error::Error + Send + Sync>)?;

	info!("Heartbeat task started. Will ping DB at {} every 30s. Hostname: {}", db_url, hostname);

	loop {
		tokio::time::sleep(Duration::from_secs(30)).await;

		let db_result = async {
			let builder = Builder::new_remote(db_url.clone(), "".to_string());
			let db = builder.build().await?;
			let conn = db.connect()?;

			let app_timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);

			let owned_hostname = hostname.clone();

			info!("Sending heartbeat ping for service 'telemetry-ingestor'");
			conn.execute(
				"INSERT INTO ServicePing (service_id, hostname, app_timestamp) VALUES (?1, ?2, ?3)",
				libsql::params!["telemetry-ingestor", owned_hostname, app_timestamp as i64],
			)
			.await?;

			Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
		}
		.await;

		if let Err(e) = db_result {
			warn!("Failed to send heartbeat ping: {}", e);
		}
	}
}

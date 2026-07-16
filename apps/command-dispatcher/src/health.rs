use std::time::Duration;

use chrono::Utc;
use libsql::{params as libsql_params, Connection};
use tracing::{info, warn};

use crate::SERVICE_ID;

pub async fn run_service_status_task(db_conn: Connection) -> Result<(), Box<dyn std::error::Error>> {
	let hostname_str = hostname::get()
		.map(|s| s.into_string().unwrap_or_else(|_| "invalid_hostname".to_string()))
		.unwrap_or_else(|_| "unknown_hostname".to_string());

	let service_instance_id = format!("{}@{}-{}", SERVICE_ID, hostname_str, std::process::id());
	let start_time_ts = Utc::now().timestamp();

	info!("Service status task started for instance: {}. Update interval: 15s", service_instance_id);

	loop {
		let current_timestamp = Utc::now().timestamp();
		let status_message = "Running and polling DB for commands".to_string();
		let current_status = "Running";

		let instance_id_clone = service_instance_id.clone();
		let hostname_clone = hostname_str.clone();

		let result = db_conn.execute(
            "INSERT INTO ServiceStatus (service_instance_id, service_name, hostname, status, status_message, last_heartbeat_at, start_time) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) \
             ON CONFLICT(service_instance_id) DO UPDATE SET service_name=excluded.service_name, hostname=excluded.hostname, status=excluded.status, status_message=excluded.status_message, last_heartbeat_at=excluded.last_heartbeat_at",
            libsql_params![
                instance_id_clone,
                SERVICE_ID,
                hostname_clone,
                current_status,
                status_message,
                current_timestamp,
                start_time_ts,
            ],
        ).await;

		if let Err(e) = result {
			warn!("Failed to update ServiceStatus for {}: DB Error: {}", service_instance_id, e);
		}

		tokio::time::sleep(Duration::from_secs(15)).await;
	}
	// Unreachable code
	// Ok(())
}

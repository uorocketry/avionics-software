use std::time::Duration;

use chrono::Utc;
use libsql::{params as libsql_params, Connection};
use serde::Serialize;
use tracing::{info, warn};

const KEEPALIVE_SOURCE: &str = "command-dispatcher-keepalive";

#[derive(Debug, Serialize)]
struct PingParameters {
	id: u32,
}

fn ping_parameters(id: u32) -> Result<String, serde_json::Error> {
	serde_json::to_string(&PingParameters { id })
}

async fn enqueue_keepalive(
	db_conn: &Connection,
	ping_id: u32,
) -> Result<bool, Box<dyn std::error::Error>> {
	let parameters = ping_parameters(ping_id)?;
	let result = db_conn
		.execute(
			"INSERT INTO OutgoingCommand (command_type, parameters, status, created_at, source_service) \
             SELECT 'Ping', ?1, 'Pending', ?2, ?3 \
             WHERE NOT EXISTS ( \
                 SELECT 1 FROM OutgoingCommand \
                 WHERE command_type = 'Ping' AND source_service = ?3 \
                   AND status IN ('Pending', 'Sending') \
             )",
			libsql_params![parameters, Utc::now().timestamp(), KEEPALIVE_SOURCE],
		)
		.await?;
	Ok(result > 0)
}

pub async fn run_keepalive_task(
	db_conn: Connection,
	interval_secs: u64,
) -> Result<(), Box<dyn std::error::Error>> {
	if interval_secs == 0 {
		info!("Automatic command keepalives are disabled");
		return Ok(());
	}

	info!("Command keepalive task started. Interval: {}s", interval_secs);
	let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
	interval.tick().await;
	let mut next_ping_id = 1_u32;

	loop {
		interval.tick().await;
		match enqueue_keepalive(&db_conn, next_ping_id).await {
			Ok(true) => {
				info!("Queued automatic Ping keepalive {}", next_ping_id);
				next_ping_id = next_ping_id.wrapping_add(1);
			}
			Ok(false) => {
				info!("Keepalive already pending; skipping this interval");
			}
			Err(error) => {
				warn!("Failed to queue automatic Ping keepalive: {}", error);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use libsql::Builder;

	use super::*;

	#[test]
	fn serializes_ping_parameters_for_the_dispatcher() {
		assert_eq!(ping_parameters(42).unwrap(), r#"{"id":42}"#);
	}

	#[test]
	fn keepalive_source_is_distinct_from_user_commands() {
		assert_eq!(KEEPALIVE_SOURCE, "command-dispatcher-keepalive");
	}

	#[tokio::test]
	async fn keeps_only_one_active_keepalive_in_the_queue() {
		let db = Builder::new_local(":memory:").build().await.unwrap();
		let connection = db.connect().unwrap();
		connection
			.execute(
				"CREATE TABLE OutgoingCommand ( \
                    id INTEGER PRIMARY KEY AUTOINCREMENT, \
                    command_type TEXT NOT NULL, \
                    parameters TEXT, \
                    status TEXT NOT NULL, \
                    created_at INTEGER NOT NULL, \
                    source_service TEXT NOT NULL \
                )",
				(),
			)
			.await
			.unwrap();

		assert!(enqueue_keepalive(&connection, 1).await.unwrap());
		assert!(!enqueue_keepalive(&connection, 2).await.unwrap());

		connection
			.execute("UPDATE OutgoingCommand SET status = 'Sent' WHERE id = 1", ())
			.await
			.unwrap();
		assert!(enqueue_keepalive(&connection, 2).await.unwrap());

		let mut rows = connection
			.query("SELECT parameters, source_service FROM OutgoingCommand ORDER BY id", ())
			.await
			.unwrap();
		let first = rows.next().await.unwrap().unwrap();
		let first_parameters = first.get::<String>(0).unwrap();
		let second = rows.next().await.unwrap().unwrap();
		let second_parameters = second.get::<String>(0).unwrap();
		let second_source = second.get::<String>(1).unwrap();
		assert_eq!(first_parameters, r#"{"id":1}"#);
		assert_eq!(second_parameters, r#"{"id":2}"#);
		assert_eq!(second_source, KEEPALIVE_SOURCE);
	}
}

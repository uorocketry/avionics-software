use libsql::{params, Connection, Result};

pub async fn save_radio_metrics(
	conn: &Connection,
	timestamp: i64,
	rssi: Option<i64>,
	packets_lost: Option<i64>,
) -> Result<()> {
	conn.execute(
		"INSERT INTO RadioMetrics (timestamp, rssi, packets_lost) VALUES (?, ?, ?)",
		params![timestamp, rssi, packets_lost],
	)
	.await?;
	Ok(())
}

use libsql::{params, Result, Transaction};
use messages_prost::{argus_state, phoenix_state};

pub async fn save_phoenix_event(
	transaction: &Transaction,
	event_value: i32,
) -> Result<i64> {
	let event_str = phoenix_state::Event::try_from(event_value)
		.map(|e| format!("{:?}", e))
		.unwrap_or_else(|_| "UNKNOWN".to_string());
	transaction
		.execute("INSERT INTO PhoenixEvent (event) VALUES (?)", params![event_str])
		.await?;
	Ok(transaction.last_insert_rowid())
}

pub async fn save_argus_event(
	transaction: &Transaction,
	event_value: i32,
) -> Result<i64> {
	let event_str = argus_state::Event::try_from(event_value)
		.map(|e| format!("{:?}", e))
		.unwrap_or_else(|_| "UNKNOWN".to_string());
	transaction
		.execute("INSERT INTO ArgusEvent (event) VALUES (?)", params![event_str])
		.await?;
	Ok(transaction.last_insert_rowid())
}

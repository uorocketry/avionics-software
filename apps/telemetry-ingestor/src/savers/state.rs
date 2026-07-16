use libsql::{params, Result, Transaction};
use messages_prost::{argus_state, phoenix_state};

pub async fn save_phoenix_state(
	transaction: &Transaction,
	state_value: i32,
) -> Result<i64> {
	let state_str = phoenix_state::State::try_from(state_value)
		.map(|s| format!("{:?}", s))
		.unwrap_or_else(|_| "UNKNOWN".to_string());
	transaction
		.execute("INSERT INTO PhoenixState (state) VALUES (?)", params![state_str])
		.await?;
	Ok(transaction.last_insert_rowid())
}

pub async fn save_argus_state(
	transaction: &Transaction,
	state_value: i32,
) -> Result<i64> {
	let state_str = argus_state::State::try_from(state_value)
		.map(|s| format!("{:?}", s))
		.unwrap_or_else(|_| "UNKNOWN".to_string());
	transaction
		.execute("INSERT INTO ArgusState (state) VALUES (?)", params![state_str])
		.await?;
	Ok(transaction.last_insert_rowid())
}

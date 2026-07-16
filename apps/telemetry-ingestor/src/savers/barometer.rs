use libsql::{params, Result, Transaction};
use messages_prost::sensor::ms5611::Barometer as Ms5611Barometer;

pub async fn save_barometer(
	transaction: &Transaction,
	baro: &Ms5611Barometer,
) -> Result<i64> {
	transaction
		.execute(
			"INSERT INTO Barometer (pressure_kpa, temperature_celsius) VALUES (?, ?)",
			params![baro.pressure_kpa, baro.temperature_celsius],
		)
		.await?;
	Ok(transaction.last_insert_rowid())
}

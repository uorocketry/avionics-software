use libsql::{params, Result, Transaction};
use messages_prost::argus::{Pressure, Strain, Temperature};

pub async fn save_argus_pressure(
	transaction: &Transaction,
	m: &Pressure,
) -> Result<i64> {
	transaction
		.execute(
			"INSERT INTO ArgusPressure (pressure, sensor_id) VALUES (?, ?)",
			params![m.pressure, m.sensor_id],
		)
		.await?;
	Ok(transaction.last_insert_rowid())
}

pub async fn save_argus_temperature(
	transaction: &Transaction,
	m: &Temperature,
) -> Result<i64> {
	transaction
		.execute(
			"INSERT INTO ArgusTemperature (temperature, sensor_id) VALUES (?, ?)",
			params![m.temperature, m.sensor_id],
		)
		.await?;
	Ok(transaction.last_insert_rowid())
}

pub async fn save_argus_strain(
	transaction: &Transaction,
	m: &Strain,
) -> Result<i64> {
	transaction
		.execute(
			"INSERT INTO ArgusStrain (strain, sensor_id) VALUES (?, ?)",
			params![m.strain, m.sensor_id],
		)
		.await?;
	Ok(transaction.last_insert_rowid())
}

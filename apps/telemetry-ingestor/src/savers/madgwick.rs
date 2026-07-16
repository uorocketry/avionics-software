use libsql::{params, Result, Transaction};
use messages_prost::madgwick::Madgwick;

pub async fn save_madgwick(
	transaction: &Transaction,
	madgwick: &Madgwick,
) -> Result<i64> {
	if let Some(quat) = &madgwick.data {
		transaction
			.execute(
				"INSERT INTO Madgwick (quat_w, quat_x, quat_y, quat_z) VALUES (?, ?, ?, ?)",
				params![quat.w, quat.x, quat.y, quat.z],
			)
			.await?;
	} else {
		transaction
			.execute(
				"INSERT INTO Madgwick (quat_w, quat_x, quat_y, quat_z) VALUES (NULL, NULL, NULL, NULL)",
				params![],
			)
			.await?;
	}
	Ok(transaction.last_insert_rowid())
}

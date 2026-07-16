use libsql::{params, Result, Transaction};
use messages_prost::command::{self as cmd};

fn node_to_string(node: i32) -> Result<String> {
	let name = messages_prost::common::Node::try_from(node)
		.map(|n| format!("{:?}", n))
		.unwrap_or_else(|_| "Unspecified".to_string());
	Ok(name)
}

fn radio_rate_to_string(rate: i32) -> &'static str {
	if rate == cmd::RadioRate::RateLow as i32 {
		"RateLow"
	} else if rate == cmd::RadioRate::RateMedium as i32 {
		"RateMedium"
	} else if rate == cmd::RadioRate::RateHigh as i32 {
		"RateHigh"
	} else {
		"RateLow"
	}
}

pub async fn save_command(
	transaction: &Transaction,
	command: &cmd::Command,
) -> Result<i64> {
	let data_type = match command.data.as_ref() {
		Some(cmd::command::Data::DeployDrogue(_)) => "DeployDrogue",
		Some(cmd::command::Data::DeployMain(_)) => "DeployMain",
		Some(cmd::command::Data::PowerDown(_)) => "PowerDown",
		Some(cmd::command::Data::PowerUpCamera(_)) => "PowerUpCamera",
		Some(cmd::command::Data::PowerDownCamera(_)) => "PowerDownCamera",
		Some(cmd::command::Data::RadioRateChange(_)) => "RadioRateChange",
		Some(cmd::command::Data::Online(_)) => "Online",
		Some(cmd::command::Data::Ping(_)) => "Ping",
		Some(cmd::command::Data::Pong(_)) => "Pong",
		None => "Unknown",
	};

	transaction
		.execute(
			"INSERT INTO Command (data_type, data_id) VALUES (?, ?)",
			params![data_type, 0], // Placeholder for data_id
		)
		.await?;
	let command_id = transaction.last_insert_rowid();

	// Save the specific Command subtype and get its ID
	let data_id: i64 = match command.data.as_ref().unwrap() {
		cmd::command::Data::DeployDrogue(deploy) => {
			transaction
				.execute("INSERT INTO DeployDrogue (val) VALUES (?)", params![deploy.val])
				.await?;
			transaction.last_insert_rowid()
		}
		cmd::command::Data::DeployMain(deploy) => {
			transaction
				.execute("INSERT INTO DeployMain (val) VALUES (?)", params![deploy.val])
				.await?;
			transaction.last_insert_rowid()
		}
		cmd::command::Data::PowerDown(power_down) => {
			transaction
				.execute("INSERT INTO PowerDown (board) VALUES (?)", params![node_to_string(power_down.board)?])
				.await?;
			transaction.last_insert_rowid()
		}
		cmd::command::Data::PowerUpCamera(_) => {
			transaction.execute("INSERT INTO PowerUpCamera DEFAULT VALUES", params![]).await?;
			transaction.last_insert_rowid()
		}
		cmd::command::Data::PowerDownCamera(_) => {
			transaction.execute("INSERT INTO PowerDownCamera DEFAULT VALUES", params![]).await?;
			transaction.last_insert_rowid()
		}
		cmd::command::Data::RadioRateChange(rate_change) => {
			transaction
				.execute(
					"INSERT INTO RadioRateChange (rate) VALUES (?)",
					params![radio_rate_to_string(rate_change.rate)],
				)
				.await?;
			transaction.last_insert_rowid()
		}
		cmd::command::Data::Online(online) => {
			transaction
				.execute("INSERT INTO Online (online) VALUES (?)", params![online.online])
				.await?;
			transaction.last_insert_rowid()
		}
		cmd::command::Data::Ping(ping) => {
			transaction.execute("INSERT INTO Ping (ping_id) VALUES (?)", params![ping.id]).await?;
			transaction.last_insert_rowid()
		}
		cmd::command::Data::Pong(pong) => {
			transaction.execute("INSERT INTO Pong (pong_id) VALUES (?)", params![pong.id]).await?;
			transaction.last_insert_rowid()
		}
	};

	// Update Command with the actual data_id
	transaction
		.execute("UPDATE Command SET data_id = ? WHERE id = ?", params![data_id, command_id])
		.await?;

	Ok(command_id)
}

use chrono::Utc;
use libsql::{params, Connection, Result, Transaction};
use messages_prost::{
	common::Node,
	radio::{self, RadioFrame},
	sbg::sbg_data,
};
use prost::Message as _;

use super::{
	argus::{save_argus_pressure, save_argus_strain, save_argus_temperature},
	barometer::save_barometer,
	command::save_command,
	event::{save_argus_event, save_phoenix_event},
	imu::save_imu,
	log::save_log,
	madgwick::save_madgwick,
	sbg::save_sbg,
	state::{save_argus_state, save_phoenix_state},
};

async fn insert_radio_message(
	transaction: &Transaction,
	node: i32,
	data_type: &str,
	data_id: i64,
	millis_since_start: Option<i64>,
) -> Result<i64> {
	let time_str = Utc::now().to_rfc3339();
	let time_epoch = Utc::now().timestamp();
	let node_name = Node::try_from(node)
		.map(|n| format!("{:?}", n))
		.unwrap_or_else(|_| "Unspecified".to_string());

	transaction
		.execute(
			"INSERT INTO RadioFrame (timestamp, timestamp_epoch, node, data_type, data_id, millis_since_start) VALUES (?, ?, ?, ?, ?, ?)",
			params![time_str, time_epoch, node_name, data_type, data_id, millis_since_start],
		)
		.await?;
	Ok(transaction.last_insert_rowid())
}

pub async fn save_messages_batch(
	db_connection: &Connection,
	message_bytes_list: Vec<Vec<u8>>,
) -> Result<()> {
	if message_bytes_list.is_empty() {
		return Ok(());
	}

	let transaction = db_connection.transaction().await?;
	tracing::info!("Starting batch save for {} messages", message_bytes_list.len());

	for message_bytes in message_bytes_list.iter() {
		match RadioFrame::decode_length_delimited(&message_bytes[..]) {
			Ok(frame) => {
				let node = frame.node;
				let millis = Some(frame.millis_since_start as i64);
				match frame.payload {
					Some(radio::radio_frame::Payload::Barometer(m)) => {
						let data_id = save_barometer(&transaction, &m).await?;
						insert_radio_message(&transaction, node, "Barometer", data_id, millis).await?;
					}
					Some(radio::radio_frame::Payload::Sbg(sbg)) => {
						if let Some(inner) = &sbg.data {
							let data_type = match inner {
								sbg_data::Data::GpsPos(_) => "SbgGpsPos",
								sbg_data::Data::UtcTime(_) => "SbgUtcTime",
								sbg_data::Data::Imu(_) => "SbgImu",
								sbg_data::Data::EkfQuat(_) => "SbgEkfQuat",
								sbg_data::Data::EkfNav(_) => "SbgEkfNav",
								sbg_data::Data::GpsVel(_) => "SbgGpsVel",
								sbg_data::Data::Air(_) => "SbgAir",
							};
							let data_id = save_sbg(&transaction, &sbg).await?;
							insert_radio_message(&transaction, node, data_type, data_id, millis).await?;
						}
					}
					// Intentionally ignored. Prefer SBG GpsPos/GpsVel.
					Some(radio::radio_frame::Payload::Gps(_gps_data)) => {
						tracing::warn!("Generic GPS payload handler removed. Prefer SBG GpsPos/GpsVel.");
					}
					Some(radio::radio_frame::Payload::Madgwick(m)) => {
						let data_id = save_madgwick(&transaction, &m).await?;
						insert_radio_message(&transaction, node, "Madgwick", data_id, millis).await?;
					}
					Some(radio::radio_frame::Payload::Iim20670(m)) => {
						let data_id = save_imu(&transaction, &m).await?;
						insert_radio_message(&transaction, node, "Imu", data_id, millis).await?;
					}
					Some(radio::radio_frame::Payload::Log(m)) => {
						let data_id = save_log(&transaction, &m).await?;
						insert_radio_message(&transaction, node, "Log", data_id, millis).await?;
					}
					Some(radio::radio_frame::Payload::PhoenixState(s)) => {
						let data_id = save_phoenix_state(&transaction, s).await?;
						insert_radio_message(&transaction, node, "PhoenixState", data_id, millis).await?;
					}
					Some(radio::radio_frame::Payload::PhoenixEvent(e)) => {
						let data_id = save_phoenix_event(&transaction, e).await?;
						insert_radio_message(&transaction, node, "PhoenixEvent", data_id, millis).await?;
					}
					Some(radio::radio_frame::Payload::ArgusState(s)) => {
						let data_id = save_argus_state(&transaction, s).await?;
						insert_radio_message(&transaction, node, "ArgusState", data_id, millis).await?;
					}
					Some(radio::radio_frame::Payload::ArgusEvent(e)) => {
						let data_id = save_argus_event(&transaction, e).await?;
						insert_radio_message(&transaction, node, "ArgusEvent", data_id, millis).await?;
					}
					Some(radio::radio_frame::Payload::ArgusPressure(m)) => {
						let data_id = save_argus_pressure(&transaction, &m).await?;
						insert_radio_message(&transaction, node, "ArgusPressure", data_id, millis).await?;
					}
					Some(radio::radio_frame::Payload::ArgusTemperature(m)) => {
						let data_id = save_argus_temperature(&transaction, &m).await?;
						insert_radio_message(&transaction, node, "ArgusTemperature", data_id, millis).await?;
					}
					Some(radio::radio_frame::Payload::ArgusStrain(m)) => {
						let data_id = save_argus_strain(&transaction, &m).await?;
						insert_radio_message(&transaction, node, "ArgusStrain", data_id, millis).await?;
					}
					Some(radio::radio_frame::Payload::Command(m)) => {
						let data_id = save_command(&transaction, &m).await?;
						insert_radio_message(&transaction, node, "Command", data_id, millis).await?;
					}
					None => {
						tracing::error!("RadioFrame had no payload. Skipping.");
					}
				}
			}
			Err(_) => {
				tracing::error!("Failed to decode RadioFrame. Skipping buffer.");
			}
		}
	}

	tracing::info!("Committing batch save");
	transaction.commit().await?;
	Ok(())
}

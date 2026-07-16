use std::time::Instant;

use chrono::Utc;
use libsql::{params as libsql_params, Connection};
use mavlink::{uorocketry::MavMessage, MavConnection, MavHeader};
use messages_prost::command as cmd;
use messages_prost::common::Node;
use messages_prost::radio::radio_frame::Payload;
use messages_prost::radio::RadioFrame;
use prost::Message as _;
use serde::Deserialize;
use tracing::{error, info};

// Struct to represent a row from the OutgoingCommand table
#[derive(Debug)]
pub struct OutgoingCommandRow {
	pub id: i64,
	pub command_type: String,
	pub parameters: Option<String>,
	pub source_service: String,
}

// Parameter structs for deserialization
#[derive(Deserialize, Debug)]
struct DeployDrogueParams {
	val: bool,
}
#[derive(Deserialize, Debug)]
struct DeployMainParams {
	val: bool,
}
#[derive(Deserialize, Debug)]
struct PowerDownParams {
	board: String,
}
#[derive(Deserialize, Debug)]
struct RadioRateChangeParams {
	rate: String,
}
#[derive(Deserialize, Debug)]
struct PingParams {
	id: u32,
}

fn parse_board(s: &str) -> Node {
	match s {
		"PressureBoard" => Node::PressureBoard,
		"StrainBoard" => Node::StrainBoard,
		"TemperatureBoard" => Node::TemperatureBoard,
		"GroundStation" => Node::GroundStation,
		"Phoenix" => Node::Phoenix,
		_ => Node::Unspecified,
	}
}

fn parse_radio_rate(s: &str) -> i32 {
	match s.to_lowercase().as_str() {
		"low" | "slow" | "0" => cmd::RadioRate::RateLow as i32,
		"medium" | "1" => cmd::RadioRate::RateMedium as i32,
		"high" | "fast" | "2" => cmd::RadioRate::RateHigh as i32,
		_ => cmd::RadioRate::RateLow as i32,
	}
}

pub async fn process_single_command(
	db_conn: &Connection,
	gateway_conn: &mut Box<dyn MavConnection<MavMessage> + Sync + Send>,
	command_row: OutgoingCommandRow,
	start_instant: Instant,
) -> Result<(), (i64, Box<dyn std::error::Error>)> {
	let cmd_id = command_row.id;
	info!(
		"[Cmd ID: {}] Processing command of type '{}' from '{}'. Params: {:?}",
		cmd_id, command_row.command_type, command_row.source_service, command_row.parameters
	);

	let queued_at_ts = Utc::now().timestamp();
	db_conn
		.execute(
			"UPDATE OutgoingCommand SET status = 'Sending', queued_at = ?, attempts = attempts + 1 WHERE id = ?",
			libsql_params![queued_at_ts, cmd_id],
		)
		.await
		.map_err(|e| (cmd_id, e.into()))?;
	info!("[Cmd ID: {}] Marked as 'Sending'.", cmd_id);

	let default_target = Node::PressureBoard as i32;

	let command_payload: cmd::Command = match command_row.command_type.as_str() {
		"DeployDrogue" => {
			let params: DeployDrogueParams = serde_json::from_str(command_row.parameters.as_deref().unwrap_or("{}"))
				.map_err(|e| (cmd_id, Box::new(e) as Box<dyn std::error::Error>))?;
			cmd::Command {
				node: default_target,
				data: Some(cmd::command::Data::DeployDrogue(cmd::DeployDrogue { val: params.val })),
			}
		}
		"DeployMain" => {
			let params: DeployMainParams = serde_json::from_str(command_row.parameters.as_deref().unwrap_or("{}"))
				.map_err(|e| (cmd_id, Box::new(e) as Box<dyn std::error::Error>))?;
			cmd::Command {
				node: default_target,
				data: Some(cmd::command::Data::DeployMain(cmd::DeployMain { val: params.val })),
			}
		}
		"PowerDown" => {
			let params_str = command_row.parameters.as_deref().ok_or_else(|| {
				(
					cmd_id,
					Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Missing parameters for PowerDown")) as Box<dyn std::error::Error>,
				)
			})?;
			let params: PowerDownParams = serde_json::from_str(params_str).map_err(|e| (cmd_id, Box::new(e) as Box<dyn std::error::Error>))?;
			let board = parse_board(&params.board) as i32;
			cmd::Command {
				node: board,
				data: Some(cmd::command::Data::PowerDown(cmd::PowerDown { board })),
			}
		}
		"PowerUpCamera" => cmd::Command {
			node: default_target,
			data: Some(cmd::command::Data::PowerUpCamera(cmd::PowerUpCamera {})),
		},
		"PowerDownCamera" => cmd::Command {
			node: default_target,
			data: Some(cmd::command::Data::PowerDownCamera(cmd::PowerDownCamera {})),
		},
		"RadioRateChange" => {
			let params_str = command_row.parameters.as_deref().ok_or_else(|| {
				(
					cmd_id,
					Box::new(std::io::Error::new(
						std::io::ErrorKind::InvalidInput,
						"Missing parameters for RadioRateChange",
					)) as Box<dyn std::error::Error>,
				)
			})?;
			let params: RadioRateChangeParams = serde_json::from_str(params_str).map_err(|e| (cmd_id, Box::new(e) as Box<dyn std::error::Error>))?;
			cmd::Command {
				node: default_target,
				data: Some(cmd::command::Data::RadioRateChange(cmd::RadioRateChange {
					rate: parse_radio_rate(&params.rate),
				})),
			}
		}
		"Ping" => {
			let id = command_row
				.parameters
				.as_deref()
				.and_then(|s| serde_json::from_str::<PingParams>(s).ok())
				.map(|p| p.id)
				.unwrap_or(0);
			cmd::Command {
				node: default_target,
				data: Some(cmd::command::Data::Ping(cmd::Ping { id })),
			}
		}
		_ => {
			let err_msg = format!("Unknown command type: {}", command_row.command_type);
			error!("[Cmd ID: {}] {}", cmd_id, err_msg);
			db_conn
				.execute(
					"UPDATE OutgoingCommand SET status = 'Failed', sent_at = ?, error_message = ? WHERE id = ?",
					libsql_params![Utc::now().timestamp(), err_msg.clone(), cmd_id],
				)
				.await
				.map_err(|e| (cmd_id, e.into()))?;
			return Err((cmd_id, err_msg.into()));
		}
	};

	let millis_since_start = start_instant.elapsed().as_millis() as u64;
	let frame = RadioFrame {
		node: Node::GroundStation as i32,
		payload: Some(Payload::Command(command_payload)),
		millis_since_start,
	};
	let bytes = RadioFrame::encode_length_delimited_to_vec(&frame);

	let mut fixed_payload = [0u8; 255];
	let len = bytes.len().min(255);
	fixed_payload[..len].copy_from_slice(&bytes[..len]);
	let send_msg = MavMessage::POSTCARD_MESSAGE(mavlink::uorocketry::POSTCARD_MESSAGE_DATA { message: fixed_payload });

	info!("[Cmd ID: {}] Sending MAVLink message...", cmd_id);
	match gateway_conn.send(&MavHeader::default(), &send_msg) {
		Ok(_) => {
			info!("[Cmd ID: {}] Successfully sent. Updating status to 'Sent'.", cmd_id);
			db_conn
				.execute(
					"UPDATE OutgoingCommand SET status = 'Sent', sent_at = ? WHERE id = ?",
					libsql_params![Utc::now().timestamp(), cmd_id],
				)
				.await
				.map_err(|e| (cmd_id, e.into()))?;
			Ok(())
		}
		Err(mav_err) => {
			let err_msg = format!("Failed to send MAVLink message: {}", mav_err);
			error!("[Cmd ID: {}] {}", cmd_id, err_msg);
			db_conn
				.execute(
					"UPDATE OutgoingCommand SET status = 'Failed', sent_at = ?, error_message = ? WHERE id = ?",
					libsql_params![Utc::now().timestamp(), err_msg.clone(), cmd_id],
				)
				.await
				.map_err(|e_db| (cmd_id, e_db.into()))?;
			Err((cmd_id, mav_err.into()))
		}
	}
}

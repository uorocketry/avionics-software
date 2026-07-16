// Messages are decoded in batch; no need to parse them here
use std::time::{Duration, Instant};

use chrono::Utc;
use libsql::Connection;
use mavlink::uorocketry::MavMessage;
use mavlink::MavConnection;
use tracing::{error, info, warn};

use crate::savers;

const BATCH_SIZE: usize = 100;
const BATCH_TIMEOUT: Duration = Duration::from_millis(500);

pub async fn handle_messages(
	mavlink_connection: Box<dyn MavConnection<MavMessage> + Send + Sync>,
	db_connection: Connection,
) -> Result<(), Box<dyn std::error::Error>> {
	info!("Getting Messages...");
	let mut last_seq_num = 0;
	let mut message_buffer: Vec<Vec<u8>> = Vec::with_capacity(BATCH_SIZE);
	let mut last_batch_time = Instant::now();

	loop {
		let db_conn_for_batch = db_connection.clone();
		let recv_result = mavlink_connection.recv();

		// info!("recv_result: {:?}", recv_result);

		match recv_result {
			Ok((header, message)) => {
				// info!("Received message: {:?}", header.sequence);
				let packets_lost = ((header.sequence as i32) - (last_seq_num as i32) - 1).rem_euclid(256);
				last_seq_num = header.sequence;

				if packets_lost > 0 {
					warn!("Packets Lost: {}", packets_lost);
					let ts = Utc::now().timestamp();
					let conn_clone = db_connection.clone();
					tokio::spawn(async move {
						if let Err(e) = savers::radio::save_radio_metrics(&conn_clone, ts, None, Some(packets_lost as i64)).await {
							error!("Failed to save packet loss metric: {:?}", e);
						}
					});
				}

				match message {
					MavMessage::POSTCARD_MESSAGE(data) => {
						// Buffer POSTCARD messages for decoding in batch
						message_buffer.push(data.message.to_vec());
					}
					MavMessage::RADIO_STATUS(data) => {
						info!("Received radio status: {:?}", data);
						let rssi_val = data.rssi as i64;
						let ts = Utc::now().timestamp();
						let conn_clone = db_connection.clone();
						tokio::spawn(async move {
							if let Err(e) = savers::radio::save_radio_metrics(&conn_clone, ts, Some(rssi_val), None).await {
								error!("Failed to save radio status: {:?}", e);
							}
						});
					}
					other => {
						error!("Received an unexpected message type {:?}", other);
						continue;
					}
				};
			}
			Err(e) => match e {
				mavlink::error::MessageReadError::Io(io_err) => match io_err.kind() {
					std::io::ErrorKind::WouldBlock => {}
					std::io::ErrorKind::UnexpectedEof => {
						error!("Connection closed unexpectedly (EOF). Shutting down receiver loop.");
						break;
					}
					_ => {
						error!("Mavlink IO Error receiving message: {:?}", io_err);
						tokio::time::sleep(Duration::from_secs(1)).await;
						continue;
					}
				},
				e => {
					error!("Mavlink Non-IO Error receiving message: {:?}", e);
					tokio::time::sleep(Duration::from_secs(1)).await;
					continue;
				}
			},
		}

		let should_save_batch = !message_buffer.is_empty() && (message_buffer.len() >= BATCH_SIZE || last_batch_time.elapsed() >= BATCH_TIMEOUT);

		if should_save_batch {
			info!("Saving batch");
			let messages_to_save = std::mem::take(&mut message_buffer);
			last_batch_time = Instant::now();

			tokio::spawn(async move {
				match savers::message::save_messages_batch(&db_conn_for_batch, messages_to_save).await {
					Ok(_) => info!("Batch saved successfully."),
					Err(e) => error!("Failed to save batch: {:?}", e),
				}
			});
			message_buffer.reserve(BATCH_SIZE);
		}
	}

	Ok(())
}

use std::error::Error;
use std::sync::mpsc;
use std::thread;

use mavlink::uorocketry::MavMessage;
use mavlink::{connect, MavConnection, MavHeader};
use tracing::{debug, error, info, warn};

pub fn send_over_mavlink(
	conn: &mut Box<dyn MavConnection<MavMessage> + Send + Sync>,
	bytes: &[u8],
) -> Result<(), Box<dyn Error>> {
	let mut fixed = [0u8; 255];
	let len = bytes.len().min(255);
	fixed[..len].copy_from_slice(&bytes[..len]);
	let msg = MavMessage::POSTCARD_MESSAGE(mavlink::uorocketry::POSTCARD_MESSAGE_DATA { message: fixed });
	conn.send(&MavHeader::default(), &msg).map(|_| ()).map_err(|e| e.into())
}

pub fn spawn_sender(connection_string: String) -> (mpsc::Sender<Vec<u8>>, thread::JoinHandle<()>) {
	let (tx, rx) = mpsc::channel::<Vec<u8>>();
	let handle = thread::spawn(move || {
		loop {
			info!("Sender thread: connecting to {}", connection_string);
			let mut conn = match connect::<MavMessage>(&connection_string) {
				Ok(c) => c,
				Err(e) => {
					warn!("Sender connect failed: {}. Retrying...", e);
					std::thread::sleep(std::time::Duration::from_secs(1));
					continue;
				}
			};

			info!("Sender connected. Waiting for outbound frames...");
			// Drain and send until error or channel closed
			loop {
				match rx.recv() {
					Ok(bytes) => {
						if let Err(e) = send_over_mavlink(&mut conn, &bytes) {
							error!("Sender send error: {}. Reconnecting...", e);
							break; // reconnect
						}
					}
					Err(_) => {
						info!("Sender channel closed. Exiting sender thread.");
						return;
					}
				}
			}
		}
	});
	(tx, handle)
}

pub fn spawn_receiver(connection_string: String) -> (mpsc::Receiver<Vec<u8>>, thread::JoinHandle<()>) {
	let (tx, rx) = mpsc::channel::<Vec<u8>>();
	let handle = thread::spawn(move || {
		loop {
			info!("Receiver thread: connecting to {}", connection_string);
			let conn = match connect::<MavMessage>(&connection_string) {
				Ok(c) => c,
				Err(e) => {
					warn!("Receiver connect failed: {}. Retrying...", e);
					std::thread::sleep(std::time::Duration::from_secs(1));
					continue;
				}
			};

			info!("Receiver connected. Listening for incoming frames...");
			loop {
				match conn.recv() {
					Ok((_header, message)) => match message {
						MavMessage::POSTCARD_MESSAGE(data) => {
							let _ = tx.send(data.message.to_vec());
						}
						MavMessage::RADIO_STATUS(_) => {
							// ignore here
						}
						other => {
							debug!("Receiver: ignoring unexpected message: {:?}", other);
						}
					},
					Err(mavlink::error::MessageReadError::Io(io_err)) if io_err.kind() == std::io::ErrorKind::WouldBlock => {
						// spin
						continue;
					}
					Err(mavlink::error::MessageReadError::Io(io_err)) if io_err.kind() == std::io::ErrorKind::UnexpectedEof => {
						warn!("Receiver: connection closed (EOF). Reconnecting...");
						break; // reconnect
					}
					Err(e) => {
						warn!("Receiver: error: {}. Reconnecting...", e);
						break; // reconnect
					}
				}
			}
		}
	});
	(rx, handle)
}

use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use mavlink::uorocketry::MavMessage;
use messages_prost::{radio, sbg::sbg_data};
use prost::Message as _;
use serde::Serialize;
use tracing::{error, info, warn};

#[derive(Parser, Debug, Clone)]
#[command(
	version,
	about = "Stream latest GPS coordinates and altitude from MAVLink POSTCARD messages as JSON lines"
)]
struct Args {
	/// MAVLink connection string (e.g., tcpout:127.0.0.1:5656, udpin:0.0.0.0:14550)
	#[arg(long, default_value = "tcpout:127.0.0.1:5656")]
	connection: String,

	/// Print pretty JSON (multi-line) instead of compact single-line JSON
	#[arg(long, default_value_t = false)]
	pretty: bool,
}

#[derive(Default, Debug, Clone, Serialize)]
struct LatestSample {
	lat: Option<f64>,
	lon: Option<f64>,
	altitude_m: Option<f64>,
	ts: String,
	source: &'static str,
}

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

	let args = Args::parse();

	let mut latest = LatestSample {
		ts: Utc::now().to_rfc3339(),
		source: "gps-ingest",
		..Default::default()
	};

	loop {
		info!("Connecting to MAVLink at {}", args.connection);
		let conn = match mavlink::connect::<MavMessage>(&args.connection) {
			Ok(c) => c,
			Err(e) => {
				error!("Failed to connect to MAVLink: {}", e);
				tokio::time::sleep(std::time::Duration::from_secs(1)).await;
				continue;
			}
		};
		info!("Connected!");

		// Receive loop; on any error, break to reconnect after 1s
		loop {
			match conn.recv() {
				Ok((_header, msg)) => match msg {
					MavMessage::POSTCARD_MESSAGE(card) => {
						// Each POSTCARD message wraps a length-delimited RadioFrame (protobuf)
						match messages_prost::radio::RadioFrame::decode_length_delimited(&card.message[..]) {
							Ok(frame) => handle_radio_frame(frame, &mut latest, args.pretty),
							Err(_) => warn!("Failed to decode RadioFrame from POSTCARD message"),
						}
					}
					MavMessage::RADIO_STATUS(status) => {
						info!("Radio status: rssi={}, remrssi={}", status.rssi, status.remrssi);
					}
					_other => {
						// Not relevant for GPS/altitude output
					}
				},
				Err(err) => match err {
					mavlink::error::MessageReadError::Io(io_err) => match io_err.kind() {
						std::io::ErrorKind::WouldBlock => {
							// No data available; yield briefly
							tokio::task::yield_now().await;
						}
						_ => {
							error!("MAVLink IO error: {:?}. Will reconnect.", io_err);
							break;
						}
					},
					other => {
						error!("MAVLink recv error: {:?}. Will reconnect.", other);
						break;
					}
				},
			}
		}

		// Reconnect delay
		tokio::time::sleep(std::time::Duration::from_secs(1)).await;
	}
}

fn handle_radio_frame(
	frame: radio::RadioFrame,
	latest: &mut LatestSample,
	pretty: bool,
) {
	use radio::radio_frame::Payload;

	match frame.payload {
		Some(Payload::Sbg(sbg)) => match sbg.data {
			Some(sbg_data::Data::GpsPos(gps_pos)) => {
				if let Some(d) = gps_pos.data {
					let lat = d.latitude;
					let lon = d.longitude;

					// Skip if lat or lon is very close to 0 (invalid GPS coordinates)
					if lat.abs() < 0.001 || lon.abs() < 0.001 {
						warn!("Invalid GPS coordinates (near 0,0): lat={}, lon={}", lat, lon);
						return;
					}

					// Check if coordinates are within Canada's bounds
					// Bounds: [minLat, minLon, maxLat, maxLon] = [41.68, -141.00, 83.11, -52.62]
					if !(41.68..=83.11).contains(&lat) || !(-141.00..=-52.62).contains(&lon) {
						warn!("GPS coordinates outside Canada bounds: lat={}, lon={}", lat, lon);
						return;
					}

					latest.lat = Some(lat);
					latest.lon = Some(lon);
					// Do not use GPS altitude; we prefer barometric altitude from SbgAir
					latest.ts = Utc::now().to_rfc3339();
					// Only output if we have all required fields
					if latest.lat.is_some() && latest.lon.is_some() && latest.altitude_m.is_some() {
						output(latest, pretty);
					}
				}
			}
			Some(sbg_data::Data::Air(air)) => {
				if let Some(d) = air.data {
					latest.altitude_m = Some(d.altitude as f64);
					latest.ts = Utc::now().to_rfc3339();
					// Only output if we have all required fields
					if latest.lat.is_some() && latest.lon.is_some() && latest.altitude_m.is_some() {
						output(latest, pretty);
					}
				}
			}
			// Other SBG data not needed for this tool
			_ => {
				// println!("other sbg");
			}
		},
		// We intentionally ignore other payloads (IMU, logs, etc.)
		_payload => {
			// println!("other payload: {:?}", _payload);
			// print payload json
			// if let Ok(s) = serde_json::to_string_pretty(&payload) {
			//     println!("{}", s);
			// }
		}
	}
}

fn output(
	sample: &LatestSample,
	pretty: bool,
) {
	// Additional validation to ensure we never output invalid data
	if sample.lat.is_none() || sample.lon.is_none() || sample.altitude_m.is_none() {
		return;
	}

	if pretty {
		if let Ok(s) = serde_json::to_string_pretty(sample) {
			println!("{}", s);
		}
	} else if let Ok(s) = serde_json::to_string(sample) {
		println!("{}", s);
	}
}

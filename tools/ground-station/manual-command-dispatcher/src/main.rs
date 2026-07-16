mod args;
mod commands;
mod messages {
	pub mod display;
}
mod transport;
mod ui;

use std::error::Error;
use std::fs::OpenOptions;
use std::io;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use args::Args;
use clap::Parser;
use commands::{build_command, MenuItem};
use crossterm::event::{self, Event as CEvent, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use messages_prost::common::Node;
use messages_prost::radio::radio_frame::Payload;
use messages_prost::radio::RadioFrame;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tracing::{error, info};
use transport::{spawn_receiver, spawn_sender};

// state, args, and menu items moved to modules

// build_command now lives in commands.rs

fn encode_message<M: prost::Message>(msg: &M) -> Vec<u8> {
	prost::Message::encode_length_delimited_to_vec(msg)
}

// drawing moved to ui::draw

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	let args = Args::parse();
	let start_instant = Instant::now();
	// Initialize logging to a file relative to current working directory
	let log_path = std::env::current_dir()?.join("manual-command-dispatcher.log");
	let log_file = OpenOptions::new().create(true).append(true).open(&log_path)?;
	let file_arc = Arc::new(Mutex::new(log_file));
	tracing_subscriber::fmt()
		.with_max_level(tracing::Level::INFO)
		.with_writer({
			let file_arc = Arc::clone(&file_arc);
			move || file_arc.lock().unwrap().try_clone().unwrap()
		})
		.init();
	info!("Starting manual-command-dispatcher. Logging to {:?}", log_path);

	// Spawn IO threads so UI is not blocked
	let (tx_out, _send_handle) = spawn_sender(args.gateway_connection_string.clone());
	let (rx_in, _recv_handle) = spawn_receiver(args.gateway_connection_string.clone());

	// Setup TUI
	enable_raw_mode()?;
	let mut stdout = io::stdout();
	crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	let mut state = ui::AppState::new(args.origin_node, args.target_node);
	let mut last_tick = Instant::now();
	let tick_rate = Duration::from_millis(200);

	loop {
		terminal.draw(|f| ui::draw(f, &state))?;

		let timeout = tick_rate.checked_sub(last_tick.elapsed()).unwrap_or(Duration::from_secs(0));

		if crossterm::event::poll(timeout)? {
			if let CEvent::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
				match code {
					KeyCode::Char('q') => break,
					KeyCode::Up => {
						if state.selected == 0 {
							state.selected = MenuItem::all().len() - 1;
						} else {
							state.selected -= 1;
						}
					}
					KeyCode::Down => {
						state.selected = (state.selected + 1) % MenuItem::all().len();
					}
					KeyCode::Enter => {
						let item = MenuItem::all()[state.selected];
						info!(
							"Enter pressed -> preparing command: {:?}, origin={:?}, target={:?}",
							item,
							Node::try_from(state.origin_node).unwrap_or(Node::Unspecified),
							Node::try_from(state.target_node).unwrap_or(Node::Unspecified)
						);
						let command = build_command(item, state.origin_node, state.target_node, &mut state.ping_counter);

						// Radio frame
						let frame = RadioFrame {
							payload: Some(Payload::Command(command)),
							node: state.origin_node,
							millis_since_start: start_instant.elapsed().as_millis() as u64,
						};

						let bytes = encode_message(&frame);
						info!("Encoded command {:?} into {} bytes", item, bytes.len());
						info!("Queueing frame to sender thread...");
						let send_len = bytes.len();
						match tx_out.send(bytes) {
							Ok(_) => {
								info!("Send success: {:?} ({} bytes)", item, send_len);
								state.left_log_lines.push(format!("Sent {:?} ({} bytes)", item, send_len));
								let sm = crate::messages::display::summarize_command(state.origin_node, &command);
								state.right_sent_messages.push(sm);
							}
							Err(e) => {
								error!("Send failed for {:?}: {}", item, e);
								state.left_log_lines.push(format!("Send error: {}", e));
							}
						}
					}
					KeyCode::Char('n') => {
						// cycle target node
						state.target_node = match Node::try_from(state.target_node).unwrap_or(Node::Unspecified) {
							Node::Unspecified => Node::PressureBoard as i32,
							Node::PressureBoard => Node::TemperatureBoard as i32,
							Node::TemperatureBoard => Node::StrainBoard as i32,
							Node::StrainBoard => Node::GroundStation as i32,
							Node::GroundStation => Node::Phoenix as i32,
							Node::Phoenix => Node::Unspecified as i32,
						};
					}
					KeyCode::Char('p') => {
						// cycle origin node
						state.origin_node = match Node::try_from(state.origin_node).unwrap_or(Node::GroundStation) {
							Node::Unspecified => Node::PressureBoard as i32,
							Node::PressureBoard => Node::TemperatureBoard as i32,
							Node::TemperatureBoard => Node::StrainBoard as i32,
							Node::StrainBoard => Node::GroundStation as i32,
							Node::GroundStation => Node::Phoenix as i32,
							Node::Phoenix => Node::Unspecified as i32,
						};
					}
					KeyCode::Char('1') if modifiers.contains(KeyModifiers::CONTROL) => {
						state.selected = 0;
					}
					KeyCode::Char('2') if modifiers.contains(KeyModifiers::CONTROL) => {
						state.selected = 1;
					}
					KeyCode::Char('3') if modifiers.contains(KeyModifiers::CONTROL) => {
						state.selected = 2;
					}
					KeyCode::Char('4') if modifiers.contains(KeyModifiers::CONTROL) => {
						state.selected = 3;
					}
					KeyCode::Char('5') if modifiers.contains(KeyModifiers::CONTROL) => {
						state.selected = 4;
					}
					KeyCode::Char('6') if modifiers.contains(KeyModifiers::CONTROL) => {
						state.selected = 5;
					}
					KeyCode::Char('7') if modifiers.contains(KeyModifiers::CONTROL) => {
						state.selected = 6;
					}
					KeyCode::Char('8') if modifiers.contains(KeyModifiers::CONTROL) => {
						state.selected = 7;
					}
					_ => {}
				}
			}
		}

		// background receive: drain a few frames per tick without blocking
		for _ in 0..16 {
			match rx_in.try_recv() {
				Ok(bytes) => {
					let summary = crate::messages::display::summarize_received_bytes(&bytes);
					state.right_received_messages.push(summary);
				}
				Err(std::sync::mpsc::TryRecvError::Empty) => break,
				Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
			}
		}

		if last_tick.elapsed() >= tick_rate {
			last_tick = Instant::now();
		}
	}

	// teardown
	disable_raw_mode()?;
	crossterm::execute!(terminal.backend_mut(), crossterm::terminal::LeaveAlternateScreen)?;
	terminal.show_cursor()?;
	Ok(())
}

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{
	atomic::{AtomicBool, AtomicU64, Ordering},
	Arc, Mutex,
};
use std::time::{Duration, Instant};

use anyhow::Result;
use crossbeam_channel as channel;
use crossterm::{
	event::{self, Event, KeyCode},
	execute,
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
	backend::CrosstermBackend,
	layout::{Constraint, Direction, Layout},
	text::{Line, Span},
	widgets::{Block, Borders, Paragraph, Wrap},
	Terminal,
};

use crate::cli::Chat;
use crate::metrics::ThroughputAverager;

pub fn run_chat(chat: Chat) -> Result<()> {
	// Connect TCP (retry until available)
	let connect = |host: std::net::SocketAddr| -> TcpStream {
		loop {
			match TcpStream::connect(host) {
				Ok(s) => {
					let _ = s.set_nodelay(true);
					let _ = s.set_nonblocking(true);
					break s;
				}
				Err(_) => {
					std::thread::sleep(Duration::from_millis(800));
				}
			}
		}
	};
	let stream = connect(chat.host);
	let stream = Arc::new(Mutex::new(stream));

	// helper to write with one retry on WouldBlock
	let try_send = |s: &mut TcpStream, data: &[u8]| -> bool {
		match s.write_all(data) {
			Ok(_) => true,
			Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
				std::thread::sleep(Duration::from_millis(50));
				s.write_all(data).is_ok()
			}
			Err(_) => false,
		}
	};

	// UI setup
	enable_raw_mode()?;
	let mut stdout = std::io::stdout();
	execute!(stdout, EnterAlternateScreen)?;
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	let stop = Arc::new(AtomicBool::new(false));
	let rx_bytes = Arc::new(AtomicU64::new(0));
	let tx_bytes = Arc::new(AtomicU64::new(0));
	let (log_tx, log_rx) = channel::unbounded::<String>();

	// Reader thread
	let stop_r = stop.clone();
	let rx_b = rx_bytes.clone();
	let rstream = Arc::clone(&stream);
	let log_tx_reader = log_tx.clone();
	std::thread::spawn(move || {
		let mut buf = [0u8; 4096];
		while !stop_r.load(Ordering::Relaxed) {
			// lock the stream for this read iteration
			let mut guard = match rstream.lock() {
				Ok(g) => g,
				Err(_) => {
					std::thread::sleep(Duration::from_millis(50));
					continue;
				}
			};
			match guard.read(&mut buf) {
				Ok(0) => {
					// EOF: server closed; reconnect proactively
					drop(guard);
					let new_s = connect(chat.host);
					if let Ok(mut g) = rstream.lock() {
						*g = new_s;
					}
					let _ = log_tx_reader.send("! reconnected".to_string());
					std::thread::sleep(Duration::from_millis(100));
				}
				Ok(n) => {
					drop(guard);
					rx_b.fetch_add(n as u64, Ordering::Relaxed);
					let s = String::from_utf8_lossy(&buf[..n]).to_string();
					let _ = log_tx_reader.send(format!("< {s}"));
				}
				Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
					drop(guard);
					std::thread::sleep(Duration::from_millis(20));
				}
				Err(_) => {
					drop(guard);
					// attempt immediate reconnect and notify
					let new_s = connect(chat.host);
					if let Ok(mut g) = rstream.lock() {
						*g = new_s;
					}
					let _ = log_tx_reader.send("! reconnected".to_string());
					std::thread::sleep(Duration::from_millis(100));
				}
			}
		}
	});

	let mut logs: Vec<String> = Vec::new();
	let mut input = String::new();
	let mut last_sent: Option<Vec<u8>> = None;
	let mut last_rx = 0u64;
	let mut last_tx = 0u64;
	let mut avg_in = ThroughputAverager::new(5.0);
	let mut avg_out = ThroughputAverager::new(5.0);
	let mut last_time = Instant::now();

	loop {
		while let Ok(line) = log_rx.try_recv() {
			logs.push(line);
			if logs.len() > 200 {
				logs.remove(0);
			}
		}

		// Throughput calc
		let now = Instant::now();
		let dt = now.duration_since(last_time).as_secs_f64().max(0.001);
		let rx = rx_bytes.load(Ordering::Relaxed);
		let tx = tx_bytes.load(Ordering::Relaxed);
		let inbound = avg_in.update(rx - last_rx, dt) as u64; // from TCP (smoothed)
		let outbound = avg_out.update(tx - last_tx, dt) as u64; // to TCP (smoothed)
		last_rx = rx;
		last_tx = tx;
		last_time = now;

		terminal.draw(|f| {
			let chunks = Layout::default()
				.direction(Direction::Vertical)
				.constraints([Constraint::Length(1), Constraint::Min(3), Constraint::Length(3)])
				.split(f.size());

			let header = Paragraph::new(format!("listener | {} | In: {} B/s Out: {} B/s", chat.host, inbound, outbound));
			f.render_widget(header, chunks[0]);

			// Auto-scroll: render only the last lines that fit
			let viewport = chunks[1].height.saturating_sub(2) as usize; // minus borders
			let start = logs.len().saturating_sub(viewport);
			let lines: Vec<Line> = logs.iter().skip(start).map(|l| Line::from(Span::raw(l.clone()))).collect();
			let para = Paragraph::new(lines)
				.wrap(Wrap { trim: false })
				.block(Block::default().title("Messages").borders(Borders::ALL));
			f.render_widget(para, chunks[1]);

			let input_box =
				Paragraph::new(input.clone()).block(Block::default().title("Input (Enter to send, Ctrl+C to quit)").borders(Borders::ALL));
			f.render_widget(input_box, chunks[2]);
		})?;

		if event::poll(Duration::from_millis(50))? {
			if let Event::Key(k) = event::read()? {
				match k.code {
					KeyCode::Char('c') if k.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => break,
					KeyCode::Char(c) => input.push(c),
					KeyCode::Backspace => {
						input.pop();
					}
					KeyCode::Enter => {
						if !input.is_empty() {
							let mut to_send = input.clone();
							to_send.push('\n');
							let mut wrote = false;
							// try write with reconnect on failure
							if let Ok(mut g) = stream.lock() {
								if let Ok(Some(_)) = g.take_error() {
									// immediate reconnect if socket error present
									let new_s = connect(chat.host);
									if let Ok(mut gg) = stream.lock() {
										*gg = new_s;
									}
								}
								wrote = try_send(&mut g, to_send.as_bytes());
								if !wrote {
									let _ = log_tx.send("! write error: Broken pipe".to_string());
								}
							}
							if !wrote {
								// reconnect and retry once
								let new_s = connect(chat.host);
								if let Ok(mut g) = stream.lock() {
									*g = new_s;
								}
								if let Ok(mut g) = stream.lock() {
									if let Some(prev) = &last_sent {
										let _ = try_send(&mut g, prev.as_slice());
									}
									std::thread::sleep(Duration::from_millis(150));
									wrote = try_send(&mut g, to_send.as_bytes());
								}
							}
							if wrote {
								tx_bytes.fetch_add(to_send.len() as u64, Ordering::Relaxed);
								let _ = log_tx.send(format!("> {input}"));
								last_sent = Some(to_send.as_bytes().to_vec());
							}
							input.clear();
						}
					}
					KeyCode::Esc => input.clear(),
					_ => {}
				}
			}
		}
	}

	stop.store(true, Ordering::Relaxed);
	disable_raw_mode()?;
	execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
	terminal.show_cursor()?;
	Ok(())
}

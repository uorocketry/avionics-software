use std::sync::{
	atomic::{AtomicBool, AtomicU64, Ordering},
	Arc,
};
use std::time::{Duration, Instant};

use anyhow::Result;
use crossbeam_channel::Receiver;
use crossterm::{
	event::{self, Event, KeyCode, KeyModifiers},
	execute,
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
	backend::CrosstermBackend,
	layout::{Constraint, Direction, Layout},
	widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
	Terminal,
};

use crate::metrics::ThroughputAverager;
use crate::state::SharedState;
use crate::ui::inspector::{DeviceId, DumpFormat, InspectorState};

#[derive(Default)]
pub struct Counters {
	pub bytes_in: AtomicU64,
	pub bytes_out: AtomicU64,
}

pub fn run_tui(
	shared: Arc<SharedState>,
	counters: Arc<Counters>,
	events: Receiver<String>,
	insp_rx: Receiver<crate::ui::inspector::Sample>,
	stop: Arc<AtomicBool>,
) -> Result<()> {
	enable_raw_mode()?;
	let mut stdout = std::io::stdout();
	execute!(stdout, EnterAlternateScreen)?;
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	let mut logs: Vec<String> = Vec::new();
	let mut log_scroll: usize = 0;
	let mut active_tab: usize = 0; // 0: Overview, 1: Inspector
	let mut _prev_tab: usize = active_tab;
	let mut insp = InspectorState::new();
	let mut last_in = 0u64;
	let mut last_out = 0u64;
	let mut avg_in = ThroughputAverager::new(5.0);
	let mut avg_out = ThroughputAverager::new(5.0);
	let mut last_time = Instant::now();

	while !stop.load(Ordering::Relaxed) {
		while let Ok(ev) = events.try_recv() {
			logs.push(ev);
			if logs.len() > 100 {
				logs.remove(0);
			}
		}

		let now = Instant::now();
		let dt = now.duration_since(last_time).as_secs_f64().max(0.001);
		let bi = counters.bytes_in.load(Ordering::Relaxed);
		let bo = counters.bytes_out.load(Ordering::Relaxed);
		let tin = avg_out.update(bi - last_in, dt) as u64; // TCP -> serial (outbound, smoothed)
		let tout = avg_in.update(bo - last_out, dt) as u64; // serial -> TCP (inbound, smoothed)
		last_in = bi;
		last_out = bo;
		last_time = now;

		// Pull inspector samples; skip if paused
		while let Ok(s) = insp_rx.try_recv() {
			if !insp.paused {
				// Track devices
				match s.dir {
					crate::ui::inspector::DirectionTag::Inbound => {
						if !insp.devices.iter().any(|d| matches!(d, DeviceId::Serial)) {
							insp.devices.insert(0, DeviceId::Serial);
						}
					}
					crate::ui::inspector::DirectionTag::Outbound(addr) => {
						if !insp.devices.iter().any(|d| matches!(d, DeviceId::Client(a) if *a == addr)) {
							insp.devices.push(DeviceId::Client(addr));
						}
					}
				}
				insp.capture.push_back(s);
				if insp.capture.len() > 4096 {
					insp.capture.pop_front();
				}
			}
		}

		terminal.draw(|f| {
			// Top-level: header tabs, main, footer
			let outer = Layout::default()
				.direction(Direction::Vertical)
				.constraints(
					[
						Constraint::Length(1), // Tabs header
						Constraint::Min(0),    // Main
						Constraint::Length(1), // Footer
					]
					.as_ref(),
				)
				.split(f.size());

			// Tabs header
			let titles = ["Overview", "Inspector"].iter().map(|t| (*t).to_string());
			let tabs = Tabs::new(titles).select(active_tab);
			f.render_widget(tabs, outer[0]);

			if active_tab == 0 {
				// Overview: connections, throughput, events
				let main = outer[1];
				let sub = Layout::default()
					.direction(Direction::Vertical)
					.constraints(
						[
							Constraint::Length(5), // Connections
							Constraint::Length(4), // Throughput
							Constraint::Min(0),    // Events
						]
						.as_ref(),
					)
					.split(main);

				let items: Vec<ListItem> = shared.tcp_connections.iter().map(|e| ListItem::new(e.key().to_string())).collect();
				let list = List::new(items).block(Block::default().title("Connections").borders(Borders::ALL));
				f.render_widget(list, sub[0]);

				let throughput = Paragraph::new(format!("Inbound: {tout} B/s\nOutbound: {tin} B/s"))
					.block(Block::default().title("Throughput").borders(Borders::ALL));
				f.render_widget(throughput, sub[1]);

				let viewport = sub[2].height.saturating_sub(2) as usize;
				let start = logs.len().saturating_sub(viewport + log_scroll);
				let log_items: Vec<ListItem> = logs.iter().skip(start).map(|l| ListItem::new(l.clone())).collect();
				let log_list = List::new(log_items).block(Block::default().title("Events").borders(Borders::ALL));
				f.render_widget(log_list, sub[2]);
			} else {
				// Inspector tab: header summary + dump list
				let main = outer[1];
				// Sidebar + main list
				let columns = Layout::default()
					.direction(Direction::Horizontal)
					.constraints(
						[
							Constraint::Length(24), // sidebar
							Constraint::Min(0),     // inspector content
						]
						.as_ref(),
					)
					.split(main);

				// Sidebar devices
				let dev_labels: Vec<String> = insp
					.devices
					.iter()
					.map(|d| match d {
						DeviceId::Serial => "serial".to_string(),
						DeviceId::Client(a) => format!("{a}"),
					})
					.collect();
				let dev_items: Vec<ListItem> = dev_labels
					.iter()
					.enumerate()
					.map(|(i, s)| {
						let prefix = if i == insp.selected { "> " } else { "  " };
						ListItem::new(format!("{prefix}{s}"))
					})
					.collect();
				let dev_list = List::new(dev_items).block(Block::default().title("Devices").borders(Borders::ALL));
				f.render_widget(dev_list, columns[0]);

				let sub = Layout::default()
					.direction(Direction::Vertical)
					.constraints(
						[
							Constraint::Length(1), // header
							Constraint::Min(0),    // list
						]
						.as_ref(),
					)
					.split(columns[1]);

				let header = Paragraph::new(format!(
					"fmt: {:?} | status: {}",
					insp.format,
					if insp.paused { "paused" } else { "resumed" }
				));
				f.render_widget(header, sub[0]);

				let para = crate::ui::inspector::inspector_paragraph(&insp, sub[1]);
				let block = Block::default().title("Messages").borders(Borders::ALL);
				f.render_widget(para.block(block), sub[1]);
			}

			// Sticky footer with keybinds
			let footer = if active_tab == 0 {
				Paragraph::new("Tab: inspector | q: quit | ↑/↓/Home: scroll events | c: clear events")
			} else {
				Paragraph::new(
					"Tab: overview | q: quit | t: toggle type | p: pause/resume | ↑/↓: select device | PgUp/PgDn: scroll | Home: top | c: clear",
				)
			};
			f.render_widget(footer, outer[2]);
		})?;

		if event::poll(Duration::from_millis(200))? {
			if let Event::Key(key) = event::read()? {
				if key.code == KeyCode::Char('q') || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL)) {
					stop.store(true, Ordering::Relaxed);
				} else if key.code == KeyCode::Tab {
					_prev_tab = active_tab;
					active_tab = (active_tab + 1) % 2;
					if active_tab == 0 {
						// leaving inspector: clear state
						insp.capture.clear();
						insp.devices = vec![DeviceId::Serial];
						insp.selected = 0;
						insp.scroll = 0;
						insp.paused = false;
					}
				} else if active_tab == 0 {
					match key.code {
						KeyCode::Up => {
							log_scroll = log_scroll.saturating_add(1);
						}
						KeyCode::Down => {
							log_scroll = log_scroll.saturating_sub(1);
						}
						KeyCode::Home => {
							log_scroll = 0;
						}
						KeyCode::Char('c') => {
							logs.clear();
							log_scroll = 0;
						}
						_ => {}
					}
				} else {
					match key.code {
						KeyCode::Char('t') => {
							insp.format = match insp.format {
								DumpFormat::Hex => DumpFormat::Ascii,
								DumpFormat::Ascii => DumpFormat::Dec,
								DumpFormat::Dec => DumpFormat::Hex,
							};
						}
						KeyCode::Char('p') => {
							insp.paused = !insp.paused;
						}
						KeyCode::Char('c') => {
							insp.capture.clear();
							insp.scroll = 0;
						}
						KeyCode::Up => {
							if insp.selected > 0 {
								insp.selected -= 1;
							}
						}
						KeyCode::Down => {
							if insp.selected + 1 < insp.devices.len() {
								insp.selected += 1;
							}
						}
						KeyCode::PageUp => {
							let limit = insp.capture.len();
							if insp.scroll < limit {
								insp.scroll = insp.scroll.saturating_add(5);
							}
						}
						KeyCode::PageDown => {
							insp.scroll = insp.scroll.saturating_sub(5);
						}
						KeyCode::Home => insp.scroll = 0,
						_ => {}
					}
				}
			}
		}
	}

	disable_raw_mode()?;
	execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
	terminal.show_cursor()?;
	Ok(())
}

use std::collections::VecDeque;
use std::net::SocketAddr;

// no time imports needed here
use bytes::Bytes;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DirectionTag {
	Inbound,
	Outbound(SocketAddr),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DeviceId {
	Serial,
	Client(SocketAddr),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DumpFormat {
	Hex,
	Ascii,
	Dec,
}

#[derive(Clone, Debug)]
pub struct Sample {
	pub dir: DirectionTag,
	pub data: Bytes,
}

pub struct InspectorState {
	pub format: DumpFormat,
	pub paused: bool,
	pub devices: Vec<DeviceId>,
	pub selected: usize,
	pub scroll: usize,
	pub capture: VecDeque<Sample>,
}

impl InspectorState {
	pub fn new() -> Self {
		Self {
			format: DumpFormat::Hex,
			paused: false,
			devices: vec![DeviceId::Serial],
			selected: 0,
			scroll: 0,
			capture: VecDeque::with_capacity(2048),
		}
	}
}

pub fn dump_bytes(
	buf: &[u8],
	fmt: DumpFormat,
	max: usize,
) -> String {
	let slice = &buf[..buf.len().min(max)];
	match fmt {
		DumpFormat::Hex => slice.iter().map(|b| format!("{b:02x} ")).collect(),
		DumpFormat::Ascii => {
			let mut s = String::new();
			for &b in slice {
				if b == b'\n' || b == b'\r' {
					continue;
				}
				if b.is_ascii_graphic() || b == b' ' {
					s.push(b as char);
				} else {
					s.push('.');
				}
			}
			s
		}
		DumpFormat::Dec => slice.iter().map(|b| format!("{b:03} ")).collect(),
	}
}

// Render wrapped text for inspector messages. Returns a Paragraph with Wrap enabled.
pub fn inspector_paragraph(
	state: &InspectorState,
	area: Rect,
) -> Paragraph<'static> {
	let filter = state.devices.get(state.selected);
	// Build lines as strings first
	let lines: Vec<String> = state
		.capture
		.iter()
		.filter_map(|s| {
			let dev = match s.dir {
				DirectionTag::Inbound => DeviceId::Serial,
				DirectionTag::Outbound(a) => DeviceId::Client(a),
			};
			if let Some(sel) = filter {
				if &dev != sel {
					return None;
				}
			}
			Some(dump_bytes(&s.data, state.format, 4096))
		})
		.collect();

	if state.scroll > 0 {
		let _ = &lines;
	}
	let start = lines.len().saturating_sub(area.height.saturating_sub(2) as usize + state.scroll);
	let visible = lines.into_iter().skip(start);

	let text_lines: Vec<Line> = visible.map(|s| Line::from(Span::raw(s))).collect();

	Paragraph::new(text_lines).wrap(Wrap { trim: false })
}

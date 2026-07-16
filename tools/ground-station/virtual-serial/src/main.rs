use std::fs;
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::io::OwnedFd;
use std::time::Duration;

use clap::Parser;
use nix::pty::{openpty, OpenptyResult, Winsize};
use tracing::info;

#[derive(Parser, Debug)]
#[command(version, about = "Creates a PTY pair and symlink at /tmp/sergw-serial")]
struct Args {}

fn create_pty_pair() -> anyhow::Result<(OwnedFd, OwnedFd, String)> {
	let ws = Winsize {
		ws_row: 24,
		ws_col: 80,
		ws_xpixel: 0,
		ws_ypixel: 0,
	};
	let OpenptyResult { master, slave } = openpty(Some(&ws), None)?;
	let slave_symlink = format!("/proc/self/fd/{}", slave.as_raw_fd());
	let slave_path = fs::read_link(&slave_symlink)?;
	Ok((master, slave, slave_path.to_string_lossy().into_owned()))
}

fn main() -> anyhow::Result<()> {
	tracing_subscriber::fmt::init();
	let _args = Args::parse();
	let (master, _slave, slave_path) = create_pty_pair()?;
	let alias = "/tmp/sergw-serial";
	let _ = fs::remove_file(alias);
	let _ = std::os::unix::fs::symlink(&slave_path, alias);
	info!("virtual serial at {} (alias {}): ready", slave_path, alias);

	// keep master open and echo periodic heartbeat so reader sees data
	let mut master_file: std::fs::File = master.into();
	loop {
		master_file.write_all(b"\n").ok();
		master_file.flush().ok();
		std::thread::sleep(Duration::from_secs(5));
		// Drain so we don't block writers
		let mut buf = [0u8; 1024];
		if let Ok(n) = master_file.read(&mut buf) {
			if n > 0 { /* ignore */ }
		}
	}
}

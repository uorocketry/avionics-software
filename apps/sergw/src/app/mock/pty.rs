#[cfg(target_os = "linux")]
use std::os::fd::AsRawFd;
#[cfg(target_os = "linux")]
use std::os::unix::io::OwnedFd;

#[cfg(target_os = "linux")]
use anyhow::Result;
#[cfg(target_os = "linux")]
use nix::pty::{openpty, OpenptyResult};

#[cfg(target_os = "linux")]
pub fn create_pty_pair() -> Result<(OwnedFd, OwnedFd, String)> {
	let OpenptyResult { master, slave, .. } = openpty(None, None)?;
	// Resolve stable path to the slave PTY for use as a serial device path
	let slave_symlink = format!("/proc/self/fd/{}", slave.as_raw_fd());
	let slave_path = std::fs::read_link(&slave_symlink)?;
	Ok((master, slave, slave_path.to_string_lossy().into_owned()))
}

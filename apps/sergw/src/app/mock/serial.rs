// Orchestrates PTY creation and UI

#[cfg(target_os = "linux")]
use anyhow::Result;

#[cfg(target_os = "linux")]
pub fn run_mock_serial() -> Result<()> {
	use super::pty::create_pty_pair;
	use super::ui::run_mock_chat_with_title;

	let (master, _slave_fd, slave_path) = create_pty_pair()?;

	// Create a default temporary alias symlink for the slave path for the program duration
	let alias_path = "/tmp/sergw-serial";
	// ensure old alias is removed, then create new symlink; cleaned up by guard on exit
	let _ = std::fs::remove_file(alias_path);
	let _ = std::os::unix::fs::symlink(&slave_path, alias_path);

	struct SymlinkGuard(&'static str);
	impl Drop for SymlinkGuard {
		fn drop(&mut self) {
			let _ = std::fs::remove_file(self.0);
		}
	}
	let _guard = SymlinkGuard(alias_path);

	run_mock_chat_with_title(master, format!("mock serial | {alias_path}"))?;
	Ok(())
}

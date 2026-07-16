use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use bytes::Bytes;
use crossbeam_channel as channel;
#[cfg(feature = "mdns")]
use libmdns as _mdns;
use tracing::{info, warn};

use crate::cli::Listen;
use crate::serial::{configure_serial, select_serial_port};
use crate::state::SharedState;
use crate::ui::inspector::{DirectionTag, Sample};
use crate::ui::overview::{run_tui, Counters};

pub fn run_listen(listen: Listen) -> Result<()> {
	let stop_flag = Arc::new(AtomicBool::new(false));
	{
		let stop = stop_flag.clone();
		let _ = ctrlc::set_handler(move || {
			stop.store(true, Ordering::Relaxed);
		});
	}

	run_listen_with_shutdown(listen, stop_flag)
}

pub(crate) fn run_listen_with_shutdown(
	listen: Listen,
	stop_flag: Arc<AtomicBool>,
) -> Result<()> {
	let serial_path = select_serial_port(&listen.serial)?;
	info!(serial = %serial_path, baud = listen.baud, host = %listen.host, "Starting sergw");
	let (status_tx, status_rx) = channel::unbounded::<String>();
	let status_tx_reader = status_tx.clone();
	let status_tx_writer = status_tx.clone();

	// Open serial with auto-reconnect loop for writer and reader handles
	let (mut serial_port, mut serial_writer_port) = open_serial_pair(&serial_path, &listen)?;

	// Channels
	// - to_serial_rx: buffers from TCP -> serial writer
	let (to_serial_tx, to_serial_rx) = channel::bounded::<Bytes>(listen.buffer);

	// - shared state for broadcasting serial -> TCP
	let shared_state = Arc::new(SharedState::new());
	let counters = Arc::new(Counters::default());
	let (event_tx_base, event_rx) = channel::unbounded::<String>();
	let event_tx = Some(event_tx_base);

	// TUI thread(s)
	let shared_for_tui = Arc::clone(&shared_state);
	let counters_for_tui = Arc::clone(&counters);
	let stop_for_tui = stop_flag.clone();
	// Inspector UI: channel
	let (insp_tx, insp_rx) = channel::bounded::<Sample>(1024);
	let status_rx_tui = status_rx.clone();
	let tui_handle = if listen.no_tui {
		None
	} else {
		Some(thread::spawn(move || {
			// Merge status messages into events
			let (tx, merged_rx) = channel::unbounded::<String>();
			let status_rx_tui_clone = status_rx_tui.clone();
			std::thread::spawn(move || loop {
				crossbeam_channel::select! {
					recv(event_rx) -> msg => if let Ok(m)=msg { let _=tx.send(m); } else { break; },
					recv(status_rx_tui_clone) -> msg => if let Ok(m)=msg { let _=tx.send(m); } else { break; },
				}
			});
			let _ = run_tui(shared_for_tui, counters_for_tui, merged_rx, insp_rx, stop_for_tui);
		}))
	};

	// Inspector receiver is moved into the TUI above; keep tx for sampling below

	// Metrics reporter (always on; logs to info every 5 seconds)
	{
		let counters_for_metrics = Arc::clone(&counters);
		let stop_for_metrics = stop_flag.clone();
		std::thread::spawn(move || {
			let mut last_in: u64 = 0; // bytes_in (TCP -> serial)
			let mut last_out: u64 = 0; // bytes_out (serial -> TCP)
			let mut last = std::time::Instant::now();
			while !stop_for_metrics.load(std::sync::atomic::Ordering::Relaxed) {
				std::thread::sleep(std::time::Duration::from_secs(5));
				let now = std::time::Instant::now();
				let dt = now.duration_since(last).as_secs_f64().max(0.001);
				last = now;
				let bi = counters_for_metrics.bytes_in.load(std::sync::atomic::Ordering::Relaxed);
				let bo = counters_for_metrics.bytes_out.load(std::sync::atomic::Ordering::Relaxed);
				let outbound = ((bi - last_in) as f64 / dt) as u64; // to serial
				let inbound = ((bo - last_out) as f64 / dt) as u64; // from serial
				last_in = bi;
				last_out = bo;
				info!(inbound_bps = inbound, outbound_bps = outbound, "Throughput");
			}
		});
	}

	// Serial reader thread: serial -> broadcast
	let shared_state_for_reader = Arc::clone(&shared_state);
	let stop_reader = stop_flag.clone();
	let serial_path_for_reader = serial_path.clone();
	let listen_for_reader = listen.clone();
	let counters_reader = Arc::clone(&counters);
	let insp_tx_reader = insp_tx.clone();
	let serial_reader = thread::spawn(move || -> Result<()> {
		let mut buffer = vec![0u8; 4096];
		loop {
			while !stop_reader.load(Ordering::Relaxed) {
				match serial_port.read(&mut buffer) {
					Ok(n) if n > 0 => {
						counters_reader.bytes_out.fetch_add(n as u64, Ordering::Relaxed);
						let _ = insp_tx_reader.try_send(Sample {
							dir: DirectionTag::Inbound,
							data: Bytes::copy_from_slice(&buffer[..n]),
						});
						let bytes = Bytes::copy_from_slice(&buffer[..n]);
						shared_state_for_reader.broadcast(bytes);
					}
					Ok(_) => {}
					Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {}
					Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
						warn!("Serial: disconnected (BrokenPipe), attempting reconnect...");
						let _ = status_tx_reader.send("Serial: disconnected, attempting reconnect...".into());
						break;
					}
					Err(e) => {
						warn!(?e, "Error reading from serial");
						break;
					}
				}
			}
			if stop_reader.load(Ordering::Relaxed) {
				break;
			}
			// Attempt reconnect every second
			match open_serial_pair(&serial_path_for_reader, &listen_for_reader) {
				Ok((sp, spw)) => {
					serial_port = sp;
					// serial writer port is owned by writer thread; we keep only reader here
					drop(spw);
					// Quiet console; status sent to UI
					let _ = status_tx_reader.send("Serial: reconnected (reader)".into());
				}
				Err(e) => {
					warn!(?e, "Reconnect failed (reader), retrying in 1s");
					std::thread::sleep(Duration::from_secs(1));
				}
			}
		}
		Ok(())
	});

	// Serial writer thread: TCP -> serial
	let stop_writer = stop_flag.clone();
	let serial_path_for_writer = serial_path.clone();
	let listen_for_writer = listen.clone();
	let serial_writer = thread::spawn(move || -> Result<()> {
		loop {
			if stop_writer.load(Ordering::Relaxed) {
				break;
			}
			match to_serial_rx.recv_timeout(Duration::from_millis(200)) {
				Ok(buf) => {
					if let Err(e) = serial_writer_port.write_all(&buf) {
						warn!(?e, "Serial: write failed, reconnecting writer...");
						let _ = status_tx_writer.send("Serial: write failed, reconnecting writer...".into());
						// try to reconnect serial writer and send a priming
						// zero-length write to ensure OS queues are ready
						loop {
							if stop_writer.load(Ordering::Relaxed) {
								return Ok(());
							}
							match open_serial_pair(&serial_path_for_writer, &listen_for_writer) {
								Ok((sp, spw)) => {
									// keep writer
									serial_writer_port = spw;
									drop(sp); // reader will reconnect separately
				   // Quiet console; status sent to UI
									let _ = status_tx_writer.send("Serial: reconnected (writer)".into());
									// After successful reconnect, retry the buffered write once
									let _ = serial_writer_port.write_all(&buf);
									let _ = serial_writer_port.flush();
									break;
								}
								Err(err) => {
									warn!(?err, "Reconnect failed (writer), retrying in 1s");
									std::thread::sleep(Duration::from_secs(1));
								}
							}
						}
					}
				}
				Err(channel::RecvTimeoutError::Timeout) => {}
				Err(channel::RecvTimeoutError::Disconnected) => break,
			}
		}
		Ok(())
	});

	// TCP acceptor
	let listener = TcpListener::bind(listen.host).with_context(|| format!("Binding TCP listener at {}", listen.host))?;
	listener.set_nonblocking(true).context("Setting TCP listener non-blocking mode")?;

	// mDNS/Bonjour advertisement (zero-config), optional via feature flag
	#[cfg(feature = "mdns")]
	let _mdns_guard: Option<(_mdns::Responder, _mdns::Service)> = {
		// Derive a friendly instance name from the serial device
		let instance = serial_path
			.rsplit('/')
			.next()
			.map(|s| format!("sergw:{s}"))
			.unwrap_or_else(|| "sergw".to_string());
		match _mdns::Responder::new() {
			Ok(responder) => {
				let port = listen.host.port();
				let txt: [&str; 1] = ["provider=sergw"];
				let service = responder.register("_sergw._tcp".to_string(), instance, port, &txt);
				Some((responder, service))
			}
			Err(e) => {
				warn!(error = ?e, "mDNS responder init failed; continuing without mDNS");
				None
			}
		}
	};

	loop {
		if stop_flag.load(Ordering::Relaxed) {
			break;
		}
		let (stream, addr) = match listener.accept() {
			Ok(conn) => conn,
			Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
				// avoid busy loop
				std::thread::sleep(Duration::from_millis(50));
				continue;
			}
			Err(e) => {
				warn!(?e, "Accept failed");
				continue;
			}
		};
		let mut stream_reader = stream.try_clone().context("Cloning TCP stream (reader)")?;
		let mut stream_writer = stream;
		if let Err(e) = stream_reader.set_nodelay(true) {
			warn!(?e, %addr, "Failed to set TCP_NODELAY on reader");
		}
		if let Err(e) = stream_writer.set_nodelay(true) {
			warn!(?e, %addr, "Failed to set TCP_NODELAY on writer");
		}
		info!(%addr, "Accepted connection");
		if let Some(tx) = &event_tx {
			let _ = tx.send(format!("Connected: {addr}"));
		}

		let to_serial_tx_conn = to_serial_tx.clone();
		let (to_tcp_tx, to_tcp_rx) = channel::bounded::<Bytes>(listen.buffer);

		// Register connection for broadcasts
		shared_state.insert(addr, to_tcp_tx);

		// Introduce connection-specific stop flag to cleanly tear down both threads if either fails
		let conn_stop = Arc::new(AtomicBool::new(false));

		// TCP reader: TCP -> to_serial
		let stop_conn = stop_flag.clone();
		let conn_stop_reader = conn_stop.clone();
		let reader_addr = addr;
		let counters_in = Arc::clone(&counters);
		let insp_tx_reader = insp_tx.clone();
		let tcp_reader = thread::spawn(move || -> Result<()> {
			let mut buffer = [0u8; 4096];
			while !stop_conn.load(Ordering::Relaxed) && !conn_stop_reader.load(Ordering::Relaxed) {
				match stream_reader.read(&mut buffer) {
					Ok(0) => break,
					Ok(n) => {
						counters_in.bytes_in.fetch_add(n as u64, Ordering::Relaxed);
						let buf = Bytes::copy_from_slice(&buffer[..n]);
						let _ = insp_tx_reader.try_send(Sample {
							dir: DirectionTag::Outbound(reader_addr),
							data: buf.clone(),
						});
						if let Err(e) = to_serial_tx_conn.send(buf) {
							warn!(?e, "Dropping data to serial, backpressure or shutdown");
							break;
						}
					}
					Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
						// Prevent 100% CPU busy loop
						std::thread::sleep(Duration::from_millis(20));
					}
					Err(e) => {
						warn!(?e, addr = %reader_addr, "TCP read error");
						break;
					}
				}
			}
			conn_stop_reader.store(true, Ordering::Relaxed);
			Ok(())
		});

		// TCP writer: from broadcast -> TCP
		let stop_conn = stop_flag.clone();
		let conn_stop_writer = conn_stop.clone();
		let writer_addr = addr;
		let tcp_writer = thread::spawn(move || -> Result<()> {
			while !stop_conn.load(Ordering::Relaxed) && !conn_stop_writer.load(Ordering::Relaxed) {
				match to_tcp_rx.recv_timeout(Duration::from_millis(200)) {
					Ok(buf) => {
						if let Err(e) = stream_writer.write_all(&buf) {
							warn!(?e, addr = %writer_addr, "TCP write error");
							break;
						}
					}
					Err(channel::RecvTimeoutError::Timeout) => {}
					Err(_e) => break,
				}
			}
			conn_stop_writer.store(true, Ordering::Relaxed);
			Ok(())
		});

		// Detach a supervisor for the connection
		let shared_state_remove = Arc::clone(&shared_state);
		let event_tx_conn = event_tx.clone();
		let conn_stop_supervisor = conn_stop.clone();
		thread::spawn(move || {
			// Wait for reader to complete (client closed or error)
			let _ = tcp_reader.join();
			// Stop writer thread immediately
			conn_stop_supervisor.store(true, Ordering::Relaxed);
			// Remove connection immediately so writers drop their sender and exit
			shared_state_remove.remove(&addr);
			if let Some(tx) = &event_tx_conn {
				let _ = tx.send(format!("Disconnected: {addr}"));
			}
			// Now wait for writer to finish draining/exit
			let _ = tcp_writer.join();
			info!(%addr, "Closed connection");
		});
	}

	// Shutdown
	info!("Shutting down");
	if let Err(e) = serial_reader.join().unwrap_or(Ok(())) {
		warn!(?e, "Serial reader error on shutdown");
	}
	if let Err(e) = serial_writer.join().unwrap_or(Ok(())) {
		warn!(?e, "Serial writer error on shutdown");
	}
	shared_state.dispose();

	if let Some(handle) = tui_handle {
		let _ = handle.join();
	}

	Ok(())
}

fn open_serial_pair(
	serial_path: &str,
	listen: &Listen,
) -> Result<(Box<dyn serialport::SerialPort>, Box<dyn serialport::SerialPort>)> {
	let builder = serialport::new(serial_path, listen.baud);
	let port = configure_serial(builder, listen).with_context(|| format!("Opening serial port {serial_path}"))?;
	let writer = port
		.try_clone()
		.with_context(|| format!("Cloning serial port {serial_path} for writer"))?;
	Ok((port, writer))
}

#[cfg(all(test, target_os = "linux"))]
mod itests {
	use std::fs::File;
	use std::io::{Read, Write};
	use std::net::TcpStream;
	use std::os::unix::io::AsRawFd;
	use std::os::unix::io::OwnedFd;
	use std::sync::atomic::AtomicBool;
	use std::thread::JoinHandle;

	use super::*;

	// Use a PTY pair to simulate a serial device. The slave path behaves like a tty device.
	fn create_pty() -> anyhow::Result<(OwnedFd, String)> {
		use nix::pty::{openpty, OpenptyResult, Winsize};
		let OpenptyResult { master, slave, .. } = openpty(None::<&Winsize>, None)?;
		// Resolve the symlink to an actual tty path
		let slave_path = format!("/proc/self/fd/{}", slave.as_raw_fd());
		let path = std::fs::read_link(&slave_path)?;
		// Drop slave (closes fd). Keep master for test I/O.
		drop(slave);
		Ok((master, path.to_string_lossy().into_owned()))
	}

	fn spawn_server(
		serial_path: String,
		host: &str,
		buffer: usize,
	) -> (JoinHandle<anyhow::Result<()>>, Arc<AtomicBool>) {
		let listen = Listen {
			serial: Some(serial_path),
			baud: 115_200,
			host: host.parse().unwrap(),
			data_bits: crate::cli::DataBitsOpt::Eight,
			parity: crate::cli::ParityOpt::None,
			stop_bits: crate::cli::StopBitsOpt::One,
			buffer,
			no_tui: true,
		};
		let stop = Arc::new(AtomicBool::new(false));
		let stop_clone = stop.clone();
		let handle = std::thread::spawn(move || run_listen_with_shutdown(listen, stop_clone));
		(handle, stop)
	}

	#[test]
	fn tcp_to_serial_and_back() {
		// Arrange: create PTY and start server bound to localhost ephemeral port
		let (master_fd, slave_path) = create_pty().expect("pty");
		let mut master: File = master_fd.into();
		let host = "127.0.0.1:6767"; // fixed test port
		let (handle, stop) = spawn_server(slave_path, host, 64);

		// connect TCP client
		std::thread::sleep(Duration::from_millis(100));
		let mut tcp = loop {
			match TcpStream::connect(host) {
				Ok(s) => break s,
				Err(_) => std::thread::sleep(Duration::from_millis(50)),
			}
		};
		tcp.set_nodelay(true).ok();

		// TCP -> serial: write to TCP, read from PTY master
		tcp.write_all(b"hello").unwrap();
		let mut serial_buf = [0u8; 5];
		master.read_exact(&mut serial_buf).unwrap();
		assert_eq!(&serial_buf, b"hello");

		// Serial -> TCP: write to PTY master, read from TCP
		master.write_all(b"world").unwrap();
		let mut tcp_buf = [0u8; 5];
		tcp.read_exact(&mut tcp_buf).unwrap();
		assert_eq!(&tcp_buf, b"world");

		// Shutdown
		stop.store(true, Ordering::Relaxed);
		let _ = handle.join().unwrap();
	}
}

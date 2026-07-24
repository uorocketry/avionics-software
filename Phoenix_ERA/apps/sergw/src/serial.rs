use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serialport::{available_ports, SerialPortType};

const BUFFER_SIZE: usize = 1024;
const RECONNECT_DELAY: Duration = Duration::from_secs(3);

pub fn list_available_ports() -> Result<Vec<String>, Box<dyn std::error::Error>> {
	let ports = available_ports()?
		.iter()
		.filter(|port| matches!(port.port_type, SerialPortType::UsbPort(_)))
		.map(|port| port.port_name.clone())
		.collect();
	Ok(ports)
}

pub fn setup_serial_port(
	serial_path: &str,
	baud_rate: u32,
) -> Result<Box<dyn serialport::SerialPort>, Box<dyn std::error::Error>> {
	let port = serialport::new(serial_path, baud_rate)
		.timeout(std::time::Duration::from_millis(100))
		.open()?;
	Ok(port)
}

pub fn spawn_serial_reader(
	mut port: Box<dyn serialport::SerialPort>,
	shared_state: &Arc<Mutex<crate::state::SharedState>>,
	shutdown_flag: &Arc<AtomicBool>,
) -> std::thread::JoinHandle<()> {
	let shared_state_clone = Arc::clone(shared_state);
	let shutdown_flag_clone = Arc::clone(shutdown_flag);

	std::thread::spawn(move || {
		let mut buffer = vec![0; BUFFER_SIZE];
		loop {
			if shutdown_flag_clone.load(Ordering::Acquire) {
				println!("Serial reader shutting down...");
				return;
			}
			match port.read(&mut buffer) {
				Ok(bytes_read) => {
					if bytes_read > 0 {
						let data = Arc::from(&buffer[..bytes_read]);
						crate::tcp_server::broadcast_data(&shared_state_clone, data);
					}
				}
				Err(e) if e.kind() == std::io::ErrorKind::TimedOut => (),
				Err(e) => {
					eprintln!("Error reading from serial port: {:?}. Closing reader thread.", e);
					return; // Exit on any other error to trigger reconnect.
				}
			}
		}
	})
}

pub fn spawn_serial_writer(
	rx: Receiver<Arc<[u8]>>,
	port_writer_handle: &Arc<Mutex<Option<Box<dyn serialport::SerialPort>>>>,
	shutdown_flag: &Arc<AtomicBool>,
) -> std::thread::JoinHandle<()> {
	let shutdown_flag_clone = Arc::clone(shutdown_flag);
	let port_writer_handle_clone = Arc::clone(port_writer_handle);

	std::thread::spawn(move || {
		println!("Serial writer started.");
		loop {
			if shutdown_flag_clone.load(Ordering::Acquire) {
				println!("Serial writer shutting down...");
				break;
			}

			match rx.recv_timeout(Duration::from_millis(100)) {
				Ok(data) => {
					let mut port_guard = port_writer_handle_clone.lock().expect("Mutex poisoned");
					if let Some(port) = port_guard.as_mut() {
						if let Err(e) = port.write_all(&data) {
							eprintln!("Serial write failed: {:?}. Data dropped.", e);
						}
					}
				}
				Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
				Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
					println!("Serial writer channel disconnected. This should not happen in normal shutdown.");
					break;
				}
			}
		}
		println!("Serial writer finished.");
	})
}

pub const RECONNECT_DELAY_DURATION: Duration = RECONNECT_DELAY;

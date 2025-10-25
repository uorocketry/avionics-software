use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use clap::{Parser, Subcommand};
use serialport::{available_ports, SerialPortType};

const BUFFER_SIZE: usize = 1024;
const TCP_LOOP_SLEEP_DURATION: Duration = Duration::from_millis(10);
const RECONNECT_DELAY: Duration = Duration::from_secs(3);

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true, arg_required_else_help = true)]
struct Cli {
	#[command(subcommand)]
	command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
	Ports,
	Listen(Listen),
}

#[derive(Parser)]
struct Listen {
	#[arg(long)]
	serial: String,
	#[arg(long, default_value_t = 57600)]
	baud: u32,
	#[arg(long, default_value = "127.0.0.1:5656")]
	host: String,
	#[arg(short, long, action)]
	verbose: bool,
}

struct SharedState {
	connections: HashMap<std::net::SocketAddr, Sender<Arc<[u8]>>>,
	verbose: bool,
}

impl SharedState {
	pub fn new(verbose: bool) -> Self {
		SharedState {
			connections: HashMap::new(),
			verbose,
		}
	}
}

fn main() {
	let cli = Cli::parse();

	match &cli.command {
		Some(Commands::Ports) => match list_available_ports() {
			Ok(ports) => println!("{}", ports.join(" ")),
			Err(e) => eprintln!("Error listing serial ports: {}", e),
		},
		Some(Commands::Listen(listen_opts)) => {
			if let Err(e) = run_gateway(listen_opts) {
				eprintln!("Application error: {}", e);
			}
		}
		None => unreachable!("Should be covered by arg_required_else_help = true"),
	}
}

fn list_available_ports() -> Result<Vec<String>, Box<dyn std::error::Error>> {
	let ports = available_ports()?
		.iter()
		.filter(|port| matches!(port.port_type, SerialPortType::UsbPort(_)))
		.map(|port| port.port_name.clone())
		.collect();
	Ok(ports)
}

fn run_gateway(opts: &Listen) -> Result<(), Box<dyn std::error::Error>> {
	let shared_state = Arc::new(Mutex::new(SharedState::new(opts.verbose)));
	let shutdown_flag = Arc::new(AtomicBool::new(false));
	let shared_port_writer = Arc::new(Mutex::new(None::<Box<dyn serialport::SerialPort>>));
	let (serial_writer_tx, serial_writer_rx) = channel::<Arc<[u8]>>();

	setup_ctrl_c_handler(&shutdown_flag)?;

	println!("Starting TCP listener on: {}", opts.host);
	let tcp_handle_task = spawn_tcp_listener(&opts.host, &shared_state, &shutdown_flag, serial_writer_tx)?;

	let serial_writer_task = spawn_serial_writer(serial_writer_rx, &shared_port_writer, &shutdown_flag);

	'reconnect_loop: loop {
		if shutdown_flag.load(Ordering::Acquire) {
			println!("Shutdown signal received, exiting main loop.");
			break 'reconnect_loop;
		}

		println!("Attempting to connect to serial port {} at {} baud...", opts.serial, opts.baud);
		let port = match setup_serial_port(&opts.serial, opts.baud) {
			Ok(p) => {
				println!("Successfully connected to serial port.");
				p
			}
			Err(e) => {
				eprintln!("Failed to open serial port: {}. Retrying in {} seconds...", e, RECONNECT_DELAY.as_secs());
				std::thread::sleep(RECONNECT_DELAY);
				continue 'reconnect_loop;
			}
		};

		// The port was opened successfully
		let port_writer_clone = port.try_clone()?;
		*shared_port_writer.lock().expect("Mutex poisoned") = Some(port_writer_clone);

		let serial_reader_task = spawn_serial_reader(port, &shared_state, &shutdown_flag);

		// Block until the reader thread exits
		serial_reader_task.join().unwrap_or_else(|e| {
			eprintln!("Serial reader thread panicked: {:?}", e);
		});

		// Connection is dead.
		*shared_port_writer.lock().expect("Mutex poisoned") = None;

		// Retry or exit
		if !shutdown_flag.load(Ordering::Acquire) {
			eprintln!("Serial connection lost. Attempting to reconnect...");
			std::thread::sleep(RECONNECT_DELAY);
		}
	}

	println!("Shutting down long-running tasks...");
	if let Err(e) = tcp_handle_task.join() {
		eprintln!("TCP listener thread panicked: {:?}", e);
	}
	if let Err(e) = serial_writer_task.join() {
		eprintln!("Serial writer thread panicked: {:?}", e);
	}

	println!("Application has shut down.");
	Ok(())
}

fn setup_serial_port(
	serial_path: &str,
	baud_rate: u32,
) -> Result<Box<dyn serialport::SerialPort>, Box<dyn std::error::Error>> {
	let port = serialport::new(serial_path, baud_rate)
		.timeout(std::time::Duration::from_millis(100))
		.open()?;
	Ok(port)
}

fn setup_ctrl_c_handler(shutdown_flag: &Arc<AtomicBool>) -> Result<(), Box<dyn std::error::Error>> {
	let shutdown_flag_clone = Arc::clone(shutdown_flag);
	ctrlc::set_handler(move || {
		println!("\nReceived Ctrl-C, initiating graceful shutdown...");
		shutdown_flag_clone.store(true, Ordering::Release);
	})?;
	Ok(())
}

fn spawn_serial_reader(
	mut port: Box<dyn serialport::SerialPort>,
	shared_state: &Arc<Mutex<SharedState>>,
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
						broadcast_data(&shared_state_clone, data);
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

fn spawn_serial_writer(
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

fn broadcast_data(
	shared_state: &Arc<Mutex<SharedState>>,
	data: Arc<[u8]>,
) {
	let state = shared_state.lock().expect("Mutex was poisoned");
	if state.verbose {
		println!(
			"Broadcasting {} bytes to {} TCP connections: {:?}",
			data.len(),
			state.connections.len(),
			data
		);
	}
	for (addr, sender) in state.connections.iter() {
		if sender.send(data.clone()).is_err() && state.verbose {
			eprintln!("Failed to send to {}, client handler will clean it up.", addr);
		}
	}
}

fn spawn_tcp_listener(
	host: &str,
	shared_state: &Arc<Mutex<SharedState>>,
	shutdown_flag: &Arc<AtomicBool>,
	serial_writer_tx: Sender<Arc<[u8]>>,
) -> Result<std::thread::JoinHandle<()>, Box<dyn std::error::Error>> {
	let listener = TcpListener::bind(host)?;
	listener.set_nonblocking(true)?;

	let shared_state_clone = Arc::clone(shared_state);
	let shutdown_flag_clone = Arc::clone(shutdown_flag);

	let handle = std::thread::spawn(move || {
		for stream_result in listener.incoming() {
			if shutdown_flag_clone.load(Ordering::Acquire) {
				println!("TCP listener shutting down...");
				break;
			}

			match stream_result {
				Ok(mut stream) => {
					let addr = stream.peer_addr().expect("Could not get peer address");
					println!("Accepted connection from: {}", addr);

					let (serial_sender, serial_receiver) = channel::<Arc<[u8]>>();
					shared_state_clone
						.lock()
						.expect("Mutex was poisoned")
						.connections
						.insert(addr, serial_sender);

					let shared_state_for_cleanup = Arc::clone(&shared_state_clone);
					let serial_writer_tx_clone = serial_writer_tx.clone();

					std::thread::spawn(move || {
						stream.set_nonblocking(true).expect("Failed to set stream to non-blocking");
						let mut tcp_buffer = [0; BUFFER_SIZE];

						loop {
							// Read from TCP to serial
							match stream.read(&mut tcp_buffer) {
								Ok(0) => break, // Connection closed by client
								Ok(n) => {
									let data = Arc::from(&tcp_buffer[..n]);
									if serial_writer_tx_clone.send(data).is_err() {
										eprintln!("Serial writer channel closed, closing connection.");
										break;
									}
								}
								Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
								Err(_) => break, // Connection error
							}

							// Read from serial to TCP
							match serial_receiver.try_recv() {
								Ok(data) => {
									if stream.write_all(&data).is_err() {
										break; // Connection error
									}
								}
								Err(std::sync::mpsc::TryRecvError::Empty) => {
									std::thread::sleep(TCP_LOOP_SLEEP_DURATION);
								}
								Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
							}
						}

						println!("Closing connection from: {}", addr);
						shared_state_for_cleanup.lock().expect("Mutex was poisoned").connections.remove(&addr);
					});
				}
				Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
					std::thread::sleep(TCP_LOOP_SLEEP_DURATION);
					continue;
				}
				Err(e) => {
					eprintln!("TCP accept error: {}. Shutting down listener.", e);
					break;
				}
			}
		}
		println!("TCP listener thread finished.");
	});

	Ok(handle)
}

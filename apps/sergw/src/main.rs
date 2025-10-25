use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};

use clap::{Parser, Subcommand};
use serialport::{available_ports, SerialPortType};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
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
	#[arg(long, action, default_value = "127.0.0.1:5656")]
	host: String,
	#[arg(short, long, action)]
	verbose: bool,
}

fn list_available_ports() -> Vec<String> {
	return available_ports()
		.unwrap_or_default()
		.iter()
		.filter(|port| matches!(port.port_type, SerialPortType::UsbPort(_)))
		.map(|port| port.port_name.clone())
		.collect::<Vec<String>>();
}

fn main() {
	let cli = Cli::parse();

	match &cli.command {
		Some(Commands::Ports) => {
			// Print available ports separated by spaces
			println!("{}", list_available_ports().join(" "));
		}
		Some(Commands::Listen(listen)) => {
			println!("Listening on {} at {} baud", listen.serial, listen.baud);

			// region:Open serial
			let port = match serialport::new(&listen.serial, listen.baud)
				.timeout(std::time::Duration::from_secs(2))
				.open()
			{
				Ok(port) => port,
				Err(e) => {
					eprintln!("Error opening serial port: {:?}", e);
					return;
				}
			};
			// endregion

			let shared_state = Arc::new(Mutex::new(SharedState::new(listen.verbose)));

			// region:Read & Broadcast

			let port_reader = port.try_clone().unwrap();

			let mut port_reader_clone = port_reader.try_clone().unwrap();
			let shared_state_clone = Arc::clone(&shared_state);
			let serial_reader = std::thread::spawn(move || {
				let mut buffer = vec![0; 1024];
				loop {
					match port_reader_clone.read(&mut buffer) {
						Ok(bytes_read) => {
							if bytes_read > 0 {
								shared_state_clone.lock().unwrap().broadcast(buffer[..bytes_read].to_vec());
							}
						}
						Err(e) if e.kind() == std::io::ErrorKind::TimedOut => (),
						Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
							eprintln!("Broken Pipe!!!");
							return e;
						}
						Err(e) => {
							eprintln!("Error reading from serial port: {:?}", e);
						}
					}
				}
			});

			// endregion

			println!("Host: {}", listen.host);

			// region:TCP Listener
			let listener = match TcpListener::bind(listen.host.clone()) {
				Ok(listener) => listener,
				Err(e) => {
					eprintln!("Error binding to TCP listener: {:?}", e);
					return;
				}
			};
			// endregion

			// region:Handle TCP connections
			let port_clone = port.try_clone().unwrap();
			let shared_state_clone = Arc::clone(&shared_state);
			let verbose = listen.verbose;
			let tcp_handle_task = std::thread::spawn(move || {
				loop {
					let (mut stream, addr) = match listener.accept() {
						Ok(conn) => {
							println!("Accepted connection from: {}", conn.1);
							conn
						}
						Err(e) => {
							eprintln!("Error accepting TCP connection: {:?}", e);
							continue;
						}
					};

					let (serial_sender, serial_receiver) = channel();
					shared_state_clone.lock().unwrap().connections.insert(addr, serial_sender);

					let mut port_clone = port_clone.try_clone().unwrap();
					let mut stream_clone = stream.try_clone().unwrap();
					let stop = Arc::new(AtomicBool::new(false));

					std::thread::spawn(move || {
						let stop_clone = stop.clone();
						let verbose_clone = verbose;
						let tcp_to_serial_task = std::thread::spawn(move || {
							loop {
								if stop_clone.load(std::sync::atomic::Ordering::Relaxed) {
									return Ok(());
								}
								let mut buffer = [0; 1024];
								match stream_clone.read(&mut buffer) {
									Ok(0) => {
										// println!("EOL");
										return Ok(());
									}
									// write buffer slice to n
									Ok(n) => match port_clone.write_all(&buffer[..n]) {
										Ok(_) => {
											if verbose_clone {
												println!("Wrote {} bytes to serial port: {:?}", n, &buffer[..n]);
											} else {
												println!("Wrote {} bytes to serial port", n);
											}
										}
										Err(e) => {
											eprintln!("Error writing to serial port: {:?}", e);
										}
									},
									Err(e) => {
										eprintln!("Error reading from TCP connection: {:?}", e);
										stop_clone.store(true, std::sync::atomic::Ordering::Relaxed);
										return Err(e);
									}
								}
							}
						});

						let stop_clone = stop.clone();
						let serial_to_tcp_task = std::thread::spawn(move || loop {
							match serial_receiver.recv() {
								Ok(data) => match stream.write_all(&data) {
									Ok(_) => {
										// println!("Wrote {} bytes to TCP connection", data.len());
									}
									Err(e) => {
										eprintln!("Error writing to TCP connection: {:?}", e);

										stop_clone.store(true, std::sync::atomic::Ordering::Relaxed);
										return Err(e);
									}
								},
								Err(e) => {
									eprintln!("Closing serial_to_tcp_task: {:?}", e);
									stop_clone.store(true, std::sync::atomic::Ordering::Relaxed);
									return Ok(());
								}
							}
						});

						match serial_to_tcp_task.join() {
							Ok(_) => (),
							Err(e) => {
								eprintln!("Error joining serial_to_tcp_task: {:?}", e);
							}
						}
						match tcp_to_serial_task.join() {
							Ok(_) => (),
							Err(e) => {
								eprintln!("Error joining tcp_to_serial_task: {:?}", e);
							}
						}
						println!("Closing connection from: {}", addr);
					});
				}
			});

			// endregion

			// Join
			serial_reader.join().unwrap();
			shared_state.lock().unwrap().dispose();
			tcp_handle_task.join().unwrap();
		}
		None => {
			Cli::parse_from(&["", "--help"]);
		}
	}
}

struct SharedState {
	connections: HashMap<std::net::SocketAddr, Sender<Vec<u8>>>,
	verbose: bool,
}

impl SharedState {
	pub fn new(verbose: bool) -> Self {
		SharedState {
			connections: HashMap::new(),
			verbose,
		}
	}

	fn dispose(&mut self) {
		for (_, sender) in self.connections.iter_mut() {
			drop(sender.to_owned())
		}
		self.connections.clear();
	}

	// Broadcast serial data to all TCP connections
	fn broadcast(
		&mut self,
		data: Vec<u8>,
	) {
		if self.verbose {
			println!(
				"Broadcasting {} bytes to {} TCP connections: {:?}",
				data.len(),
				self.connections.len(),
				data
			);
		}
		let mut to_remove = Vec::new();
		for (addr, serial_sender) in self.connections.iter_mut() {
			match serial_sender.send(data.clone()) {
				Ok(_) => {
					if self.verbose {
						println!("Sent {} bytes to TCP connection {}", data.len(), addr);
					}
				}
				Err(e) => {
					eprintln!("Error Broadcasting data to TCP connection: {:?}", e);
					eprintln!("Removing connection from shared state");
					to_remove.push(addr.clone());
				}
			}
		}

		for sender in to_remove {
			self.connections.remove(&sender);
		}
	}
}

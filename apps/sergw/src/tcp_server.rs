use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::state::SharedState;

const BUFFER_SIZE: usize = 1024;
const TCP_LOOP_SLEEP_DURATION: Duration = Duration::from_millis(10);

pub fn broadcast_data(
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

pub fn spawn_tcp_listener(
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

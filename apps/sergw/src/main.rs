use std::sync::atomic::AtomicBool;
use std::sync::mpsc::channel;
use std::sync::Arc;

use clap::{Parser, Subcommand};

mod serial;
mod state;
mod tcp_server;

use serial::{list_available_ports, setup_serial_port, spawn_serial_reader, spawn_serial_writer, RECONNECT_DELAY_DURATION};
use state::SharedState;
use tcp_server::spawn_tcp_listener;

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

fn setup_ctrl_c_handler(shutdown_flag: &Arc<AtomicBool>) -> Result<(), Box<dyn std::error::Error>> {
	let shutdown_flag_clone = Arc::clone(shutdown_flag);
	ctrlc::set_handler(move || {
		println!("\nReceived Ctrl-C, initiating graceful shutdown...");
		shutdown_flag_clone.store(true, std::sync::atomic::Ordering::Release);
	})?;
	Ok(())
}

fn run_gateway(opts: &Listen) -> Result<(), Box<dyn std::error::Error>> {
	let shared_state = Arc::new(std::sync::Mutex::new(SharedState::new(opts.verbose)));
	let shutdown_flag = Arc::new(AtomicBool::new(false));
	let shared_port_writer = Arc::new(std::sync::Mutex::new(None::<Box<dyn serialport::SerialPort>>));
	let (serial_writer_tx, serial_writer_rx) = channel::<Arc<[u8]>>();

	setup_ctrl_c_handler(&shutdown_flag)?;

	println!("Starting TCP listener on: {}", opts.host);
	let tcp_handle_task = spawn_tcp_listener(&opts.host, &shared_state, &shutdown_flag, serial_writer_tx)?;

	let serial_writer_task = spawn_serial_writer(serial_writer_rx, &shared_port_writer, &shutdown_flag);

	'reconnect_loop: loop {
		if shutdown_flag.load(std::sync::atomic::Ordering::Acquire) {
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
				eprintln!(
					"Failed to open serial port: {}. Retrying in {} seconds...",
					e,
					RECONNECT_DELAY_DURATION.as_secs()
				);
				std::thread::sleep(RECONNECT_DELAY_DURATION);
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
		if !shutdown_flag.load(std::sync::atomic::Ordering::Acquire) {
			eprintln!("Serial connection lost. Attempting to reconnect...");
			std::thread::sleep(RECONNECT_DELAY_DURATION);
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

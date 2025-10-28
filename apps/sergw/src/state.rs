use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::mpsc::Sender;
use std::sync::Arc;

pub struct SharedState {
	pub connections: HashMap<SocketAddr, Sender<Arc<[u8]>>>,
	pub verbose: bool,
}

impl SharedState {
	pub fn new(verbose: bool) -> Self {
		SharedState {
			connections: HashMap::new(),
			verbose,
		}
	}
}

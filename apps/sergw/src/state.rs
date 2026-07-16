use std::net::SocketAddr;

use bytes::Bytes;
use crossbeam_channel as channel;
use dashmap::DashMap;

pub struct SharedState {
	// outbound to TCP, concurrent map to avoid global mutex during broadcast
	pub tcp_connections: DashMap<SocketAddr, channel::Sender<Bytes>>,
}

impl SharedState {
	pub fn new() -> Self {
		Self {
			tcp_connections: DashMap::new(),
		}
	}

	pub fn insert(
		&self,
		addr: SocketAddr,
		tx: channel::Sender<Bytes>,
	) {
		self.tcp_connections.insert(addr, tx);
	}

	pub fn remove(
		&self,
		addr: &SocketAddr,
	) {
		self.tcp_connections.remove(addr);
	}

	pub fn dispose(&self) {
		self.tcp_connections.clear();
	}

	pub fn broadcast(
		&self,
		data: Bytes,
	) {
		// Clone senders without holding any global lock; DashMap provides
		// per-bucket locking which is brief during iteration.
		let snapshot: Vec<(SocketAddr, channel::Sender<Bytes>)> = self.tcp_connections.iter().map(|e| (*e.key(), e.value().clone())).collect();

		let mut to_remove: Vec<SocketAddr> = Vec::new();
		for (addr, tx) in snapshot.into_iter() {
			match tx.try_send(data.clone()) {
				Ok(()) => {}
				Err(channel::TrySendError::Full(_)) => {
					// Slow client: drop this client to enforce backpressure
					to_remove.push(addr);
				}
				Err(channel::TrySendError::Disconnected(_)) => {
					to_remove.push(addr);
				}
			}
		}

		for addr in to_remove {
			self.remove(&addr);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn broadcast_removes_dead_receivers() {
		let (tx_alive, rx_alive) = channel::bounded::<Bytes>(1);
		let (tx_dead, _rx_dead) = channel::bounded::<Bytes>(1);
		drop(_rx_dead); // drop to simulate dead receiver

		let state = SharedState::new();
		let a1: SocketAddr = "127.0.0.1:10000".parse().unwrap();
		let a2: SocketAddr = "127.0.0.1:10001".parse().unwrap();
		state.insert(a1, tx_alive);
		state.insert(a2, tx_dead);

		state.broadcast(Bytes::from_static(b"hello"));

		// Alive should receive
		assert_eq!(rx_alive.recv().unwrap(), Bytes::from_static(b"hello"));
		// Dead should be removed
		assert!(!state.tcp_connections.contains_key(&a2));
	}

	#[test]
	fn broadcast_removes_slow_receivers_on_full() {
		let (tx_alive, rx_alive) = channel::bounded::<Bytes>(1);
		let (tx_slow, _rx_slow) = channel::bounded::<Bytes>(1);

		let state = SharedState::new();
		let a_alive: SocketAddr = "127.0.0.1:11000".parse().unwrap();
		let a_slow: SocketAddr = "127.0.0.1:11001".parse().unwrap();
		state.insert(a_alive, tx_alive);
		state.insert(a_slow, tx_slow);

		// First broadcast fills both queues
		state.broadcast(Bytes::from_static(b"one"));

		// Drain the alive receiver so it won't be full for the next broadcast
		assert_eq!(rx_alive.recv().unwrap(), Bytes::from_static(b"one"));

		// Second broadcast: slow stays full and should be removed; alive receives
		state.broadcast(Bytes::from_static(b"two"));

		assert_eq!(rx_alive.recv().unwrap(), Bytes::from_static(b"two"));
		assert!(!state.tcp_connections.contains_key(&a_slow));
	}

	#[test]
	fn broadcast_delivers_to_multiple_alive_receivers() {
		let (tx1, rx1) = channel::unbounded::<Bytes>();
		let (tx2, rx2) = channel::unbounded::<Bytes>();

		let state = SharedState::new();
		let a1: SocketAddr = "127.0.0.1:12000".parse().unwrap();
		let a2: SocketAddr = "127.0.0.1:12001".parse().unwrap();
		state.insert(a1, tx1);
		state.insert(a2, tx2);

		state.broadcast(Bytes::from_static(b"abc"));

		assert_eq!(rx1.recv().unwrap(), Bytes::from_static(b"abc"));
		assert_eq!(rx2.recv().unwrap(), Bytes::from_static(b"abc"));
	}

	#[test]
	fn dispose_clears_all_connections() {
		let (tx1, _rx1) = channel::unbounded::<Bytes>();
		let (tx2, _rx2) = channel::unbounded::<Bytes>();
		let state = SharedState::new();
		let a1: SocketAddr = "127.0.0.1:13000".parse().unwrap();
		let a2: SocketAddr = "127.0.0.1:13001".parse().unwrap();
		state.insert(a1, tx1);
		state.insert(a2, tx2);

		state.dispose();
		assert!(state.tcp_connections.is_empty());
	}
}

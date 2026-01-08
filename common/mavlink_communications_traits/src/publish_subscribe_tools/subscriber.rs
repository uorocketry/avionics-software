use core::marker::{Send, Sync};

use mavlink::MAVLinkV2MessageRaw;
pub trait Subscriber: Send + Sync {
	/// This method updates the subscriber and is updated by mavlink periodically. See `start_subscribers` in the `initialize_subscriber_pool` macro
	fn __update(
		self: &mut Self,
		frame: MAVLinkV2MessageRaw,
	);
}

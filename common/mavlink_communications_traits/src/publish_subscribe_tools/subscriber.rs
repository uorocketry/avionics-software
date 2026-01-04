use core::marker::{Send, Sync};

use mavlink::MAVLinkV2MessageRaw;
pub trait Subscriber: Send + Sync {
	/// Returns whether the subscriber wanted the frame
	fn subscribe(
		self: &mut Self,
		_: MAVLinkV2MessageRaw,
	) -> bool;
}

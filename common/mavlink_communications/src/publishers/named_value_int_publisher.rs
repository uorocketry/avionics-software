use embassy_time::Instant;
use mavlink::{MAVLinkV2MessageRaw, MavHeader, common::NAMED_VALUE_INT_DATA, types::CharArray};
use mavlink_communications_macros::Publisher;
use mavlink_communications_traits::publish_subscribe_tools::publisher::Publisher;
use utils::data_structures::ring_buffer::RingBuffer;

#[derive(Publisher)]
pub struct NamedValueIntPublisher<const N: usize> {
    buffer: RingBuffer<NAMED_VALUE_INT_DATA, N>,
}

impl<const N: usize> NamedValueIntPublisher<N> {
    pub fn new(buffer: RingBuffer<NAMED_VALUE_INT_DATA, N>) -> NamedValueIntPublisher<N> {
        NamedValueIntPublisher {
            buffer: RingBuffer::new(),
        }
    }

    pub fn push(&mut self, mut value: NAMED_VALUE_INT_DATA, index: u8) {
        value.time_boot_ms = Instant::now().as_millis() as u32;
        self.buffer.push(value);
    }
}

use core::clone;
use core::fmt::Error;

use defmt::info;
use embassy_executor::task;
use embassy_time::{Duration, Timer};
use embedded_io_async::{ErrorType, Read, ReadExactError, Write};
use peripheral_services::serial::service::{SerialService, SerialServiceRx, SerialServiceTx};
use peripheral_services::serial_ring_buffered::service::{RingBufferedSerialService, RingBufferedSerialServiceRx, RingBufferedSerialServiceTx};
use ublox::UbxPacketRequest;
use ublox::cfg_msg::CfgMsgAllPortsBuilder;
use ublox::cfg_prt::{DataBits, InProtoMask, OutProtoMask, Parity, StopBits, UartPortId};
use ublox::esf_raw::EsfRaw;
use ublox::mon_ver::MonVer;
use ublox::packetref_proto31::PacketOwned;
use ublox::proto31::Proto31;
use ublox::{self, FixedBuffer, Parser, ParserBuilder, cfg_msg::CfgMsgSinglePort, cfg_prt::CfgPrtUartBuilder};
use ublox::{
	UbxPacket, UbxPacketCreator, UbxPacketMeta, UbxProtocol,
	cfg_prt::{CfgPrtUart, UartMode},
};
use utils::types::AsyncMutex;

use crate::max_m10m_service::traits::MaxM10MListener;

const PARSER_BUFFER_SIZE: usize = 2048;
const INTERNAL_BUFFER_SIZE: usize = 1024;

pub struct MaxM10MService {
	ubx_parser: Parser<FixedBuffer<PARSER_BUFFER_SIZE>>,
	// pub internal_buffer: [u8; INTERNAL_BUFFER_SIZE],
}

impl MaxM10MService {
	pub fn new(io_service: SerialService) -> (Self, MaxM10MTx, MaxM10MRx) {
		let parser = ParserBuilder::new().with_fixed_buffer::<PARSER_BUFFER_SIZE>();
		let (tx_serial, rx_serial) = io_service.split();

		let (tx_maxm10, rx_maxm10) = (
			MaxM10MTx { component: tx_serial },
			MaxM10MRx {
				component: rx_serial,
				internal_buffer: [0; INTERNAL_BUFFER_SIZE],
			},
		);

		return (
			MaxM10MService {
				ubx_parser: parser,
				// internal_buffer: [0; INTERNAL_BUFFER_SIZE],
			},
			tx_maxm10,
			rx_maxm10,
		);
	}

	pub fn update_parser(
		&mut self,
		new_data: &mut [u8],
	) {
		self.ubx_parser.consume_ubx(new_data);
	}

	// pub async fn poll_message<T: UbxPacketMeta>(&mut self) -> Option<PacketOwned> {
	// 	// TODO: Look into how the serial buffer is behaving, maybe drop the ring buffer

	// 	self.write(&UbxPacketRequest::request_for::<T>().into_packet_bytes()).await;
	// 	self.read_message().await
	// }

	// TODO: This needs to be changed to use serde & a custom solution. The current method is hacky. The hacky solution also does not work as expected.
	// pub async fn read_message(&mut self) -> Option<PacketOwned> {
	// 	// self.update_internal().await;
	// 	let mut parser_ref = self.ubx_parser.consume_ubx(&mut self.internal_buffer);

	// 	let mut frame: Option<PacketOwned> = None;

	// 	if let Some(Ok(UbxPacket::Proto31(packet))) = parser_ref.next() {
	// 		frame = Some(packet.to_owned());
	// 	}
	// 	let mut last_frame = None;

	// 	// This drains the internal buffer of the parser and stores the last frame in the buffer (TODO: This is hacky)
	// 	while frame.is_some() {
	// 		last_frame = frame;
	// 		if let Some(Ok(UbxPacket::Proto31(packet))) = parser_ref.next() {
	// 			frame = Some(packet.to_owned());
	// 		} else {
	// 			frame = None
	// 		}
	// 	}
	// 	last_frame
	// }

	// // pub async fn read(
	// 	&mut self,
	// 	buf: &mut [u8],
	// ) -> usize {
	// 	// match self.io_service.rx_component.read(buf).await {
	// 	// 	Ok(bytes_read) => {
	// 	// 		return bytes_read;
	// 	// 	}
	// 	// 	Err(_) => 0,
	// 	// }
	// }

	// pub async fn write(
	// 	&mut self,
	// 	buf: &[u8],
	// ) {
	// 	self.io_service.tx_component.component.write(buf).await;
	// }
}

pub struct MaxM10MTx {
	pub component: SerialServiceTx,
}

impl MaxM10MTx {
	pub async fn configure(
		&mut self,
		configurator: CfgPrtUartBuilder,
	) {
		self.write(&configurator.into_packet_bytes()).await;
	}

	pub async fn set_rates<T: UbxPacketMeta>(
		&mut self,
		rates: [u8; 6],
	) {
		self.write(&CfgMsgAllPortsBuilder::set_rate_for::<T>(rates).into_packet_bytes()).await;
	}

	pub async fn update_message<T: UbxPacketMeta>(&mut self) {
		// TODO: Look into how the serial buffer is behaving, maybe drop the ring buffer
		self.write(&UbxPacketRequest::request_for::<T>().into_packet_bytes()).await;
	}
}
impl ErrorType for MaxM10MTx {
	type Error = embassy_stm32::usart::Error;
}

impl Write for MaxM10MTx {
	async fn write(
		&mut self,
		buf: &[u8],
	) -> Result<usize, Self::Error> {
		self.component.write(buf).await
	}
}
pub struct MaxM10MRx {
	pub component: SerialServiceRx,
	pub internal_buffer: [u8; INTERNAL_BUFFER_SIZE],
}

impl ErrorType for MaxM10MRx {
	type Error = embassy_stm32::usart::Error;
}

impl Read for MaxM10MRx {
	async fn read(
		&mut self,
		buf: &mut [u8],
	) -> Result<usize, Self::Error> {
		self.component.read(buf).await
	}

	async fn read_exact(
		&mut self,
		mut buf: &mut [u8],
	) -> Result<(), embedded_io_async::ReadExactError<Self::Error>> {
		while !buf.is_empty() {
			match self.component.read_exact(buf).await {
				Ok(_) => break,
				Err(e) => return Err(e),
			}
		}
		if buf.is_empty() {
			Ok(())
		} else {
			Err(embedded_io_async::ReadExactError::UnexpectedEof)
		}
	}
}

pub fn start_maxm10m(
	spawner: &embassy_executor::Spawner,
	service: &'static AsyncMutex<MaxM10MService>,
	mut rx_component: MaxM10MRx,
	listeners: &'static mut [&'static AsyncMutex<dyn MaxM10MListener>],
	delay: Duration,
) {
	spawner.spawn(start_maxm10m_read(service, rx_component));
	spawner.spawn(start_maxm10m_periodic(service, listeners, delay));
}

#[task]
async fn start_maxm10m_read(
	service: &'static AsyncMutex<MaxM10MService>,
	mut rx_component: MaxM10MRx,
) {
	loop {
		_ = rx_component.component.read(&mut rx_component.internal_buffer).await;
		service.lock().await.update_parser(&mut rx_component.internal_buffer);
	}
}

#[task]
pub async fn start_maxm10m_periodic(
	service: &'static AsyncMutex<MaxM10MService>,

	listeners: &'static mut [&'static AsyncMutex<dyn MaxM10MListener>],
	delay: Duration,
) {
	loop {
		{
			let parser = &mut service.lock().await.ubx_parser;
			let mut message_iter = parser.consume_ubx(&[]);
			let mut message = message_iter.next();
			while message.is_some() {
				// Literally no fucking clue why it is "ref packet", that is just what the compiler said XD
				if let Some(Ok(UbxPacket::Proto31(ref packet))) = message {
					for listener in listeners.iter().clone() {
						listener.lock().await.__update(&packet);
					}
				}
				message = message_iter.next();
			}
		}
		Timer::after(delay).await;
		// info!("Updating")
	}
}

CfgPrtUartBuilder {
				portid: UartPortId::Uart1,
				reserved0: 0,
				tx_ready: 0,
				mode: UartMode::new(DataBits::Eight, Parity::None, StopBits::One),
				baud_rate: 9600,
				in_proto_mask: InProtoMask::UBLOX,
				out_proto_mask: OutProtoMask::union(OutProtoMask::NMEA, OutProtoMask::UBLOX),
				flags: 0,
				reserved5: 0,
			}
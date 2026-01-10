use proc_macro::TokenStream;
use quote::quote;
use syn::Ident;
// TODO: Can probably change this to one function, rather than deciding between two
pub fn select_value_mode_stream(
    name: &Ident,
    value_field: &Ident,
    timestamp_override: bool,
) -> TokenStream {
    if timestamp_override == true {
        return __value_mode_stream_ts_override(name, value_field);
    } else {
        return __value_mode_stream(name, value_field);
    }
}
pub fn select_buffered_mode_stream(
    generic_ident: &Ident,
    name: &Ident,
    value_field: &Ident,
    timestamp_override: bool,
    burst_mode: bool,
) -> TokenStream {
    if burst_mode {
        if timestamp_override == true {
            return __buffered_mode_stream_ts_override(generic_ident, name, value_field);
        } else {
            return __buffered_burst_mode_stream(generic_ident, name, value_field);
        }
    } else {
        if timestamp_override == true {
            return __buffered_mode_stream_ts_override(generic_ident, name, value_field);
        } else {
            return __buffered_mode_stream(generic_ident, name, value_field);
        }
    }
}

fn __value_mode_stream(name: &Ident, value_field: &Ident) -> TokenStream {
    quote! {
		impl mavlink_communications_traits::publish_subscribe_tools::publisher::Publisher for #name {
		fn publish(
			&mut self,
			buff: &mut [u8],
			sequence: u8,
		) -> usize {
			let mut frame = mavlink::MAVLinkV2MessageRaw::new();
			let header = mavlink::MavHeader {
				system_id: 1,
				component_id: 1,
				sequence: sequence,
			};


			frame.serialize_message_data(header, &self.#value_field);

			let frame_size = frame.raw_bytes().len();

			buff[0..frame_size].copy_from_slice(frame.raw_bytes());
			return frame_size;
		}
	}
		}
	.into()
}

fn __value_mode_stream_ts_override(name: &Ident, value_field: &Ident) -> TokenStream {
    quote! {
		impl mavlink_communications_traits::publish_subscribe_tools::publisher::Publisher for #name {
		fn publish(
			&mut self,
			buff: &mut [u8],
			sequence: u8,
		) -> usize {
			let mut frame = mavlink::MAVLinkV2MessageRaw::new();
			let header = mavlink::MavHeader {
				system_id: 1,
				component_id: 1,
				sequence: sequence,
			};

			let payload = self.#value_field.clone();
			payload.time_boot_ms = Instant::now().as_millis() as u32;

			frame.serialize_message_data(header, payload);

			let frame_size = frame.raw_bytes().len();

			buff[0..frame_size].copy_from_slice(frame.raw_bytes());
			return frame_size;
		}
	}
		}
	.into()
}

fn __buffered_mode_stream(generic_ident: &Ident, name: &Ident, rb_ident: &Ident) -> TokenStream {
    quote! {
		impl<const #generic_ident: usize> mavlink_communications_traits::publish_subscribe_tools::publisher::Publisher for #name<#generic_ident> {
		fn publish(
			&mut self,
			buff: &mut [u8],
			sequence: u8,
		) -> usize {
			let mut frame = mavlink::MAVLinkV2MessageRaw::new();
			let mut payload;
			let header = mavlink::MavHeader {
				system_id: 1,
				component_id: 1,
				sequence: sequence,
			};
			// Checks if there is data in the publisher buffer
			match self.#rb_ident.pop() {
				Ok(value) => payload = value,
				Err(_) => {
					return 0;
				}
			}

			frame.serialize_message_data(header, &payload);

			let frame_size = frame.raw_bytes().len();

			buff[0..frame_size].copy_from_slice(frame.raw_bytes());
			return frame_size;
		}
	}
		}
	.into()
}
fn __buffered_mode_stream_ts_override(
    generic_ident: &Ident,
    name: &Ident,
    rb_ident: &Ident,
) -> TokenStream {
    quote! {
		impl<const #generic_ident: usize> mavlink_communications_traits::publish_subscribe_tools::publisher::Publisher for #name<#generic_ident> {
		fn publish(
			&mut self,
			buff: &mut [u8],
			sequence: u8,
		) -> usize {
			let mut frame = mavlink::MAVLinkV2MessageRaw::new();
			let mut payload;
			let header = mavlink::MavHeader {
				system_id: 1,
				component_id: 1,
				sequence: sequence,
			};
			// Checks if there is data in the publisher buffer
			match self.#rb_ident.pop() {
				Ok(value) => payload = value,
				Err(_) => {
					return 0;
				}
			}
			payload.time_boot_ms = Instant::now().as_millis() as u32;

			frame.serialize_message_data(header, &payload);

			let frame_size = frame.raw_bytes().len();

			buff[0..frame_size].copy_from_slice(frame.raw_bytes());
			return frame_size;
		}
	}
		}
	.into()
}

// TODO: UNIMPLEMENTED
pub fn __buffered_burst_mode_stream(
    generic_ident: &Ident,
    name: &Ident,
    rb_ident: &Ident,
) -> TokenStream {
    quote! {
		impl<const #generic_ident: usize> mavlink_communications_traits::publish_subscribe_tools::publisher::Publisher for #name<#generic_ident> {
		fn publish(
			&mut self,
			buff: &mut [u8],
			sequence: u8,
		) -> usize {
			let mut frame = mavlink::MAVLinkV2MessageRaw::new();
			let mut payload;
			let header = mavlink::MavHeader {
				system_id: 1,
				component_id: 1,
				sequence: sequence,
			};
			// Checks if there is data in the publisher buffer
			match self.#rb_ident.pop() {
				Ok(value) => payload = value,
				Err(_) => {
					return 0;
				}
			}

			

			frame.serialize_message_data(header, &payload);

			let frame_size = frame.raw_bytes().len();

			buff[0..frame_size].copy_from_slice(frame.raw_bytes());
			return frame_size;
		}
	}
		}
	.into()
}

use proc_macro::TokenStream;
use quote::quote;
use syn::Ident;

pub fn subscriber_stream(
	generic_ident: &Ident,
	name: &Ident,
	rb_ident: &Ident,
	name_of_type: &Ident,
) -> TokenStream {
	quote! {
			use mavlink_core::MessageData;
			impl<const #generic_ident: usize> crate::uor_mavlink::uor_mavlink_communications_traits::subscriber::Subscriber for #name<#generic_ident> {
				fn __update(
					self: &mut Self,
					frame: crate::uor_mavlink::uor_mavlink_dialect::MAVLinkV2MessageRaw,
					)
				{
				// Check if the subscriber subscribes to the frame
				if frame.message_id() == #name_of_type::ID {
					let payload_raw = frame.payload();
					let payload_deserialized = #name_of_type::deser(crate::uor_mavlink::uor_mavlink_dialect::MavlinkVersion::V2, payload_raw);
					if let Ok(result) = payload_deserialized {
						_ = self.#rb_ident.push(result);
					}
				}
			}
		}

			impl<const #generic_ident: usize> #name<#generic_ident> {
				pub fn pop(
					self: &mut Self,
					) -> Result<#name_of_type, uor_utils::utils::data_structures::ring_buffer::BufferError>
				{
				self.#rb_ident.pop()
			}
		}
	}
	.into()
}

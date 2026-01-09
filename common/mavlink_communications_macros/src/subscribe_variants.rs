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
            impl<const #generic_ident: usize> mavlink_communications_traits::publish_subscribe_tools::subscriber::Subscriber for #name<#generic_ident> {
                fn __update(
                    self: &mut Self,
                    frame: mavlink::MAVLinkV2MessageRaw,
                    )
                {
                // Check if the subscriber subscribes to the frame
                if frame.message_id() == #name_of_type::ID {
                    let payload_raw = frame.payload();
                    let payload_deserialized = #name_of_type::deser(mavlink::MavlinkVersion::V2, payload_raw);
                    if let Ok(result) = payload_deserialized {
                        _ = self.#rb_ident.push(result);
                    }
                }
            }
        }

            impl<const #generic_ident: usize> #name<#generic_ident> {
                pub fn pop(
                    self: &mut Self,
                    ) -> Result<#name_of_type, utils::data_structures::ring_buffer::BufferError>
                {
                self.#rb_ident.pop()
            }
        }
    }.into()
}

extern crate proc_macro;
use mavlink_communications_traits::publish_subscribe_tools::publisher::Publisher;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, GenericParam, Type, parse_macro_input};
// TODO: Use helper attribute macros to enable more features (burst mode for multi-frame, timestamp overwrites, value based, buffer based, etc)
// TODO: This macro currently has no way of detecting (and rejecting) structs with multiple ring buffers. Add this
#[proc_macro_derive(Publisher)]
pub fn derive_publish(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;

	// Extract generics from unknown type
	let generics = &input.generics;
	let mut generic_ident = None;

	for i in &generics.params {
		if let GenericParam::Const(param) = i {
			generic_ident = Some(param.ident.clone());
		}
	}

	if generic_ident.is_none() {
		return TokenStream::from(syn::Error::new(input.ident.span(), "This type lacks the buffer size constant generic").to_compile_error());
	}

	// Check that derive macro is applied to struct
	let data;
	if let Data::Struct(struct_data) = &input.data {
		data = struct_data;
	} else {
		return TokenStream::from(syn::Error::new(input.ident.span(), "This derive macro must be applied to a struct").to_compile_error());
	}

	// Check that named fields are present
	let fields;
	if let Fields::Named(struct_fields) = &data.fields {
		fields = struct_fields;
	} else {
		return TokenStream::from(
			syn::Error::new(
				input.ident.span(),
				"This derive macro must be applied to a struct who posses named fields",
			)
			.to_compile_error(),
		);
	}

	// Check if a ring buffer is present, and look for it's name
	let mut rb_ident = None;
	for i in &fields.named {
		if let Type::Path(type_data) = &i.ty {
			for x in &type_data.path.segments {
				if x.ident == "RingBuffer" {
					rb_ident = i.ident.clone()
				}
			}
		}
		println!("{:#?}", i.ty);
	}

	let output = quote! {
		impl<const #generic_ident: usize> mavlink_communications_traits::publish_subscribe_tools::publisher::Publisher for #name<#generic_ident> {
		fn publish(
			&mut self,
			buff: &mut [u8],
			sequence: u8,
		) -> usize {
			// TODO: Check that mavlink is actually freed once function is done. In theory, the function should terminate and the lock will be released, but I lowkey have a feeling it won't XD
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

			// [0..frame_size].copy_from_slice(frame.raw_bytes());
			return frame_size;
		}
	}
		};
	output.into()
}

#[proc_macro_attribute]
pub fn configure_publisher(
	attr: TokenStream,
	item: TokenStream,
) -> TokenStream {
	item
}

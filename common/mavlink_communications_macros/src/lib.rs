extern crate proc_macro;
use std::{path::is_separator, process::id};

use mavlink_communications_traits::publish_subscribe_tools::publisher::Publisher;
use proc_macro::{Ident, TokenStream};
use proc_macro2::Group;
use quote::{quote, quote_spanned};
use syn::{
	Data, DeriveInput, Expr, Fields, GenericParam, LitBool, Meta, MetaList, MetaNameValue, Type, parenthesized, parse, parse_macro_input, parse2,
	token::Static,
};

mod variants;
use variants::*;
mod utils;
use utils::*;
// TODO: Either move some of this shit into a macro or a function as the definition is getting way too long
// TODO: Check if fields provided to attribute macro actually exist
/// Derive macro for the `Publisher` trait <br>
/// Accepts `configure_publisher` as a attribute helper macro
#[proc_macro_derive(Publisher, attributes(configure_publisher))]
pub fn derive_publish(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;

	let mut burst_mode = false;
	let mut is_buffered = true;
	let mut data_field = None;
	let mut override_timestamp = true;

	println!("{:#?}", &input);

	// Parse macro attributes
	let attrs = &input.attrs;
	for i in attrs {
		if let Meta::List(meta_data) = &i.meta {
			let tokens = meta_data.tokens.clone();
			let mut tree = tokens.into_iter();

			// START OF ATTRIBUTE CHECKS

			// Check if burst mode is enabled
			let result = parse_expression_bool(&tree, "burst_mode");
			if let Err(error) = result {
				return error;
			} else {
				if let Some(value) = result.unwrap() {
					burst_mode = value;
				}
			}

			// check and apply buffer/value field override
			let result = parse_expression_ident(&tree, "data_field");
			if let Err(error) = result {
				return error;
			} else {
				data_field = result.unwrap();
			}

			// Check if buffered mode is enabled/disabled
			let result = parse_expression_bool(&tree, "is_buffered");
			if let Err(error) = result {
				return error;
			} else {
				if let Some(value) = result.unwrap() {
					is_buffered = value;
				}
			}

			// Check if publisher-set timestamp should be overridden
			let result = parse_expression_bool(&tree, "override_timestamp");
			if let Err(error) = result {
				return error;
			} else {
				if let Some(value) = result.unwrap() {
					override_timestamp = value;
				}
			}

			// END OF ATTRIBUTE CHECKS
		}
	}

	// Check that derive macro is applied to struct
	let data;
	if let Data::Struct(struct_data) = &input.data {
		data = struct_data;
	} else {
		return TokenStream::from(syn::Error::new(input.ident.span(), "This derive macro must be applied to a struct").to_compile_error());
	}

	// Extract generics from unknown struct
	let generics = &input.generics;
	let mut generic_ident = None;

	for i in &generics.params {
		if let GenericParam::Const(param) = i {
			generic_ident = Some(param.ident.clone());
		}
	}

	if generic_ident.is_none() && is_buffered {
		return TokenStream::from(syn::Error::new(input.ident.span(), "This type lacks the buffer size constant generic").to_compile_error());
	}

	// TODO: All below should be moved into a function to allow for easier configuration
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

	// If no manual buffer name is present, check if a ring buffer is present and look for it's name
	if data_field == None && is_buffered {
		for i in &fields.named {
			if let Type::Path(type_data) = &i.ty {
				for x in &type_data.path.segments {
					if x.ident == "RingBuffer" {
						data_field = i.ident.clone()
					}
				}
			}
		}
	}
	let output;

	if burst_mode && is_buffered {
		output = __buffered_burst_mode_stream(generic_ident.as_ref().unwrap(), name, &data_field.as_ref().unwrap());
	} else if !is_buffered && data_field == None {
		output = TokenStream::from(
			syn::Error::new(
				get_attr_span(&input).unwrap(),
				"\"data_field\" must be populated when not using buffered mode",
			)
			.to_compile_error(),
		);
	} else if !is_buffered {
		output = select_value_mode_stream(name, &data_field.as_ref().unwrap(), override_timestamp);
	} else {
		output = select_buffered_mode_stream(generic_ident.as_ref().unwrap(), name, &data_field.as_ref().unwrap(), override_timestamp);
	}
	output.into()
}

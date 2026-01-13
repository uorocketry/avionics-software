extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{Data, DeriveInput, Fields, GenericArgument, GenericParam, Meta, PathArguments, Type, parse_macro_input};

mod publish_variants;
use publish_variants::*;
mod subscribe_variants;
use subscribe_variants::*;
mod utils;
use utils::*;

const TYPE_IDENTIFIER_INDEX: usize = 0;
// TODO: Either move some of this shit into a macro or a function as the definition is getting way too long
/// Derive macro for the `Publisher` trait <br>
/// Accepts `configure_publisher` as a attribute helper macro
#[proc_macro_derive(Publisher, attributes(configure_publisher))]
pub fn derive_publish(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name: &syn::Ident = &input.ident;

	let mut burst_mode = false;
	let mut is_buffered = true;
	let mut data_field = None;
	let mut override_timestamp = true;

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
		return TokenStream::from(syn::Error::new(input.ident.span(), "This struct lacks the buffer size constant generic").to_compile_error());
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

	// If no manual buffer name is present, check if a ring buffer is present and look for it's name
	if data_field == None && is_buffered {
		for i in &fields.named {
			if let Type::Path(type_data) = &i.ty {
				for x in &type_data.path.segments {
					if x.ident == "RingBuffer" {
						data_field = i.ident.clone();
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
		output = select_buffered_mode_stream(
			generic_ident.as_ref().unwrap(),
			name,
			&data_field.as_ref().unwrap(),
			override_timestamp,
			burst_mode,
		);
	}
	output.into()
}

#[proc_macro_derive(Subscriber, attributes(configure_subscriber))]
pub fn derive_subscribe(input: TokenStream) -> TokenStream {
	// TODO: Ensure that all variables take a reference to input, rather than copying/cloning
	let input = parse_macro_input!(input as DeriveInput);
	let name: &syn::Ident = &input.ident;

	// let frame_type;

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

	if generic_ident.is_none() {
		return TokenStream::from(syn::Error::new(input.ident.span(), "This struct lacks the buffer size constant generic").to_compile_error());
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

	let mut name_of_type: Option<proc_macro2::Ident> = None;
	let mut rb_ident = None;
	for i in &fields.named {
		if let Type::Path(type_data) = &i.ty {
			for x in &type_data.path.segments {
				if x.ident == "RingBuffer" {
					rb_ident = i.ident.clone();
					// Look for the type
					// TODO: move this into a function
					if let PathArguments::AngleBracketed(arguments) = x.arguments.clone() {
						// Index of the type
						// TODO: Get rid of this magic number, (0 is the index of the type name in the array arguments)
						let arg = arguments.args.get(TYPE_IDENTIFIER_INDEX);
						if let None = arg {
							return TokenStream::from(
								syn::Error::new(input.ident.span(), "Array type is malformed. Insure that the type is defined").to_compile_error(),
							);
						}
						// TODO: Add error stuff for this block:
						if let Some(GenericArgument::Type(Type::Path(path))) = arg {
							// idfk, its needed to "chisel the ice" into the type
							// TODO: Get rid of this magic number, (0 is the index of the type name in the array arguments)
							if let Some(type_path) = path.path.segments.get(TYPE_IDENTIFIER_INDEX) {
								name_of_type = Some(type_path.ident.clone());
							} else {
								return TokenStream::from(
									syn::Error::new(input.ident.span(), "Array type is malformed. Insure that the type is defined")
										.to_compile_error(),
								);
							}
						} else {
							return TokenStream::from(
								syn::Error::new(input.ident.span(), "Array type is malformed. Insure that the type is defined").to_compile_error(),
							);
						}
						// TODO: END OF BLOCK
					}
				}
			}
		}
	}

	// Unwrap on "name_of_type" should always resolve to Some(x), and is only used to get around the need to initialize the variable
	let output = subscriber_stream(&generic_ident.unwrap(), name, &rb_ident.unwrap(), &name_of_type.unwrap());
	return output.into();
}

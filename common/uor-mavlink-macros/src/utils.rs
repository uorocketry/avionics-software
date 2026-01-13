use proc_macro::TokenStream;
use proc_macro2::extra::DelimSpan;
use proc_macro2::{Ident, Punct, token_stream::IntoIter};
use proc_macro2::{Span, TokenTree};
use quote::{ToTokens, quote_spanned};
use syn::parse::Parse;
use syn::{DeriveInput, Error, Lit};
use syn::{parse_str, token};

pub fn parse_expression_bool(
	iterator: &IntoIter,
	key: &str,
) -> Result<Option<bool>, TokenStream> {
	let mut iterator = iterator.clone().peekable();
	// Extract key
	while let Some(value) = iterator.next() {
		// Converts the value to the string, and tries to parse it as a identifier token
		let ident: Result<Ident, syn::Error> = parse_str(value.to_string().as_str());
		if let Ok(identifier) = ident {
			if identifier.to_string() == key {
				// Check that the expression isn't malformed
				let punct = iterator.peek().unwrap();
				if let Ok(_) = parse_str(punct.to_string().as_str()) as Result<Punct, syn::Error> {
					// Advance iterator and check the value of the expression
					iterator.next();
					let value_token = iterator.peek().unwrap();
					if let Ok(Lit::Bool(result)) = parse_str(value_token.to_string().as_str()) {
						return Ok(Some(result.value()));
					} else {
						return Err(quote_spanned! {
							value_token.span() => compile_error!("value is missing from expression");
						}
						.into());
					}
				} else {
					return Err(quote_spanned! {
						punct.span() => compile_error!("expression is malformed");
					}
					.into());
				}
			}
		}
	}
	println!("Could not find key {:?}", key);
	// Default value
	return Ok(None);
}

pub fn parse_expression_string(
	iterator: &IntoIter,
	key: &str,
) -> Result<Option<String>, TokenStream> {
	let mut iterator = iterator.clone().peekable();
	// Extract key
	while let Some(value) = iterator.next() {
		// Converts the value to the string, and tries to parse it as a identifier token
		let ident: Result<Ident, syn::Error> = parse_str(value.to_string().as_str());
		if let Ok(identifier) = ident {
			if identifier.to_string() == key {
				// Check that the expression isn't malformed
				let punct = iterator.peek().unwrap();
				if let Ok(_) = parse_str(punct.to_string().as_str()) as Result<Punct, syn::Error> {
					// Advance iterator and check the value of the expression
					iterator.next();
					let value_token = iterator.peek().unwrap();
					if let Ok(Lit::Str(result)) = parse_str(value_token.to_string().as_str()) {
						return Ok(Some(result.value()));
					} else {
						return Err(quote_spanned! {
							value_token.span() => compile_error!("value is missing from expression");
						}
						.into());
					}
				} else {
					return Err(quote_spanned! {
						punct.span() => compile_error!("expression is malformed");
					}
					.into());
				}
			}
		}
	}
	println!("Could not find key {:?}", key);
	// Default value
	return Ok(None);
}

pub fn parse_expression_ident(
	iterator: &IntoIter,
	key: &str,
) -> Result<Option<Ident>, TokenStream> {
	let mut iterator = iterator.clone().peekable();
	// Extract key
	while let Some(value) = iterator.next() {
		// Converts the value to the string, and tries to parse it as a identifier token
		let ident: Result<Ident, syn::Error> = parse_str(value.to_string().as_str());
		if let Ok(identifier) = ident {
			if identifier.to_string() == key {
				// Check that the expression isn't malformed
				let punct = iterator.peek().unwrap();
				if let Ok(_) = parse_str(punct.to_string().as_str()) as Result<Punct, syn::Error> {
					// Advance iterator and check the value of the expression
					iterator.next();
					let value_token = iterator.peek().unwrap();
					if let Ok(result) = parse_str(value_token.to_string().as_str()) as Result<Ident, syn::Error> {
						return Ok(Some(result));
					} else {
						return Err(quote_spanned! {
							value_token.span() => compile_error!("value is missing from expression");
						}
						.into());
					}
				} else {
					return Err(quote_spanned! {
						punct.span() => compile_error!("expression is malformed");
					}
					.into());
				}
			}
		}
	}
	println!("Could not find key {:?}", key);
	// Default value
	return Ok(None);
}

pub fn get_attr_span(tokens: &DeriveInput) -> Option<Span> {
	let tokens = tokens.attrs.get(0)?;
	Some(tokens.bracket_token.span.join())
}

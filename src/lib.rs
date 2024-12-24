//! A prototype notation for referentially stable constructors
//!
//! ## Tasks
//!
//! - [ ] support structs
//! - [ ] support enums
//! - [ ] support traits
//! - [ ] support custom drop impls

#![forbid(unsafe_code)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs)]

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod inherent_impl;
mod strukt;
mod utils;

/// Enable methods to be constructed and operate in-place
#[proc_macro_attribute]
pub fn spati(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::Item);
    match item {
        syn::Item::Impl(item) => inherent_impl::process_impl(item),
        syn::Item::Struct(item) => strukt::process_struct(item),
        _ => quote::quote! {
            compile_error!("`[spati, E0001] invalid item kind: macro `spati` can only be used on inherent impls and structs");
        }.into()
    }
}

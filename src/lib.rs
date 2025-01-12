//! A prototype notation for referentially stable constructors
//!
//! ## Why the name?
//!
//! A placing is a typical east-German convenience store. It's a staple in Berlin,
//! and it doesn't seem to be going anywhere. Just like the values created by
//! this crate.
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

mod inherent;
mod strukt;
mod utils;

/// Enable methods to be constructed and operate in-place.
#[proc_macro_attribute]
pub fn placing(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::Item);
    match item {
        syn::Item::Impl(item) => inherent::process_impl(item),
        syn::Item::Struct(item) => strukt::process_struct(item),
        _ => quote::quote! {
            compile_error!("`[placing, E0001] invalid item kind: macro `placing` can only be used on inherent impls and structs");
        }.into()
    }
}

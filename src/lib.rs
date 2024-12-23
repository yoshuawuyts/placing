//! A prototype notation for placement new

#![forbid(unsafe_code)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute};

mod attr;

// return quote::quote_spanned! {
//     attr.span() => compile_error!("`[spati, E0001] wrong item kind,\nmacro `spati` can only be used on `impl {{}}` blocks");
// }
// .into();

#[proc_macro_attribute]
pub fn spati(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr: attr::Attributes = syn::parse(attr).unwrap();
    let item = parse_macro_input!(item as syn::Item);
    let item = match item {
        syn::Item::Impl(impl_item) => impl_item,
        _ => {
            return quote::quote! {
                compile_error!("`[spati, E0001] wrong item kind: macro `spati` can only be used on `impl {}` blocks");
            }
            .into();
        }
    };
    // println!("item: \"{item:#?}\"");
    let expanded = quote! {#item};
    expanded.into()
}

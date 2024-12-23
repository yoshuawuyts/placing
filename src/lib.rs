//! A prototype notation for placement new

#![forbid(unsafe_code)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs)]

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod inherent_impl;

// return quote::quote_spanned! {
//     attr.span() => compile_error!("`[spati, E0001] wrong item kind,\nmacro `spati` can only be used on `impl {{}}` blocks");
// }
// .into();

/// Enable methods to be constructed and operate in-place
#[proc_macro_attribute]
pub fn spati(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::Item);
    match item {
        syn::Item::Impl(impl_item) => inherent_impl::process_impl(impl_item),
        _ => quote::quote! {
            compile_error!("`[spati, E0001] wrong item kind: macro `spati` can only be used on `impl {}` blocks");
        }.into()
    }
}

use proc_macro::TokenStream;
use syn::{spanned::Spanned, ImplItemFn};

use crate::utils;

use super::ImplFns;

mod inline;
mod pointer;

/// Rewrite a `#[placing]` statement to create the inner type instead
pub(crate) fn rewrite_placing_constructor(
    output: &mut ImplFns,
    f: ImplItemFn,
    ident: &syn::Ident,
) -> Result<(), TokenStream> {
    match utils::constructor_type(&f.sig, ident) {
        utils::ConstructorKind::Inline => inline::inline_constructor(output, f),
        utils::ConstructorKind::Pointer(kind) => pointer::pointer_constructor(output, f, kind),
        utils::ConstructorKind::Other => {
            return Err(quote::quote_spanned! { f.sig.output.span() =>
                compile_error!("[E0009, placing] invalid constructor return type"),
            }
            .into())
        }
    }
}

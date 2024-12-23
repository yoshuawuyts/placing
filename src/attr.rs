use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Ident, Result, Token};

// Parses a unit struct with attributes.
pub(crate) struct Attributes {
    attrs: Vec<syn::Attribute>,
}

impl Attributes {
    pub(crate) fn span(&self) -> Span {
        self.attrs[0].span()
    }
}

impl Parse for Attributes {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Attributes {
            attrs: input.call(syn::Attribute::parse_outer)?,
        })
    }
}

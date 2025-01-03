use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, Attribute, GenericParam, Path, Token};

/// Add the const param to the trait definition
pub(crate) fn create_maybe_generics(generics: &syn::Generics) -> syn::Generics {
    let mut outer_generics = generics.clone();
    let params = &mut outer_generics.params;
    if !params.empty_or_trailing() {
        params.push_punct(<Token![,]>::default());
    }
    let param = syn::parse2(quote! { const EMPLACE: bool = false }).unwrap();
    params.push(GenericParam::Const(param));
    outer_generics
}

/// Checks whether there is a `#[super]` attribute in the list.
///
/// # Errors
///
/// This function will return an error if the attribute is malformed
pub(crate) fn has_super(attrs: &[Attribute]) -> Result<bool, TokenStream> {
    for attr in attrs.iter() {
        if path_ident(attr.path()) != "super" {
            continue;
        }

        return match &attr.meta {
            syn::Meta::Path(_) => Ok(true),
            _ => Err(quote::quote_spanned! { attr.span() =>
                compile_error!("[E0004, spati] invalid attr: the #[super] attribute does not support any additional arguments"),
            }.into()),
        };
    }
    Ok(false)
}

/// Convert a path to its identity
pub(crate) fn path_ident(path: &Path) -> String {
    path.to_token_stream().to_string()
}

pub(crate) fn strip_super(attrs: &mut Vec<Attribute>) {
    attrs.retain(|attr| path_ident(attr.path()) != "super")
}

pub(crate) fn set_path_generics(
    path: &syn::TypePath,
    base: &syn::Generics,
    param: syn::GenericArgument,
) -> syn::TypePath {
    let mut path = path.clone();
    let segment = path.path.segments.last_mut().unwrap();
    let ident = &segment.ident;
    let params = base.params.iter().map(|param| -> syn::GenericArgument {
        match param {
            syn::GenericParam::Lifetime(lifetime_param) => {
                let param = &lifetime_param.lifetime;
                syn::parse2(quote! {#param}).unwrap()
            }
            syn::GenericParam::Type(type_param) => {
                let param = &type_param.ident;
                syn::parse2(quote! {#param}).unwrap()
            }
            syn::GenericParam::Const(const_param) => {
                let param = &const_param.ident;
                syn::parse2(quote! {#param}).unwrap()
            }
        }
    });
    *segment = syn::parse2(quote! { #ident <#(#params,)* #param> }).unwrap();
    path
}

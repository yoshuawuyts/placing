use quote::quote;
use syn::{GenericParam, Token};

/// Add the const param to the trait definition
pub(crate) fn create_outer_generics(generics: &syn::Generics) -> syn::Generics {
    let mut outer_generics = generics.clone();
    let params = &mut outer_generics.params;
    if !params.empty_or_trailing() {
        params.push_punct(<Token![,]>::default());
    }
    let param = syn::parse2(quote! { const EMPLACE: bool = false }).unwrap();
    params.push(GenericParam::Const(param));
    outer_generics
}

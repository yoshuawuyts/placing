use quote::quote;
use syn::{ConstParam, GenericParam, Token};

/// Add the const param to the trait definition
pub(crate) fn create_outer_generics(generics: &syn::Generics) -> syn::Generics {
    let mut outer_generics = generics.clone();
    let params = &mut outer_generics.params;
    if !params.empty_or_trailing() {
        params.push_punct(<Token![,]>::default());
    }
    params.push(create_const_param());
    outer_generics
}

fn create_const_param() -> GenericParam {
    let item = quote! { const EMPLACE: bool = false };
    let item: ConstParam = syn::parse2(item).unwrap();
    GenericParam::Const(item)
}

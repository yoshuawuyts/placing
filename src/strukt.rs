use proc_macro::TokenStream;

use quote::{format_ident, quote};
use syn::{parse, parse_macro_input, ConstParam, GenericParam, ItemStruct, Token, Type};

/// Process an impl block that carries the `#[spati]` notation
pub(crate) fn process_struct(item: ItemStruct) -> TokenStream {
    // We need all the impl components to later recreate it
    // and fill it with our own methods
    let ItemStruct {
        attrs: _,
        vis,
        struct_token,
        ident,
        generics,
        fields,
        semi_token,
    } = item;

    // Add the const param to the trait definition
    let mut outer_generics = generics.clone();
    let params = &mut outer_generics.params;
    if !params.empty_or_trailing() {
        params.push_punct(<Token![,]>::default());
    }
    params.push(create_const_param());

    let inner_ident = format_ident!("Inner{}", ident);

    // TODO: correctly quote all generic args

    quote! {
        #vis #struct_token #ident #outer_generics
        #semi_token
        (::core::mem::MaybeUninit<#inner_ident #generics>);

        #struct_token #inner_ident #generics
        #semi_token
        #fields

        impl<const EMPLACE: bool> ::core::ops::Deref for #ident<EMPLACE> {
            type Target = #inner_ident;
            fn deref(&self) -> &Self::Target {
                unsafe { self.0.assume_init_ref() }
            }
        }

        impl<const EMPLACE: bool> ::core::ops::DerefMut for #ident<EMPLACE> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                unsafe { self.0.assume_init_mut() }
            }
        }

        impl<const EMPLACE: bool> Drop for #ident<EMPLACE> {
            fn drop(&mut self) {
                unsafe { self.0.assume_init_drop() }
            }
        }
    }
    .into()
}

fn create_const_param() -> GenericParam {
    let item = quote! { const EMPLACE: bool = false };
    let item: ConstParam = syn::parse2(item).unwrap();
    GenericParam::Const(item)
}

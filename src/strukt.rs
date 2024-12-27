use proc_macro::TokenStream;

use quote::{format_ident, quote};
use syn::ItemStruct;

use crate::utils::create_outer_generics;

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

    let outer_generics = create_outer_generics(&generics);
    let (outer_impl, outer_ty, outer_where) = outer_generics.split_for_impl();

    let inner_ident = format_ident!("Inner{}", ident);
    let (inner_impl, inner_ty, inner_where) = generics.split_for_impl();

    quote! {
        #vis #struct_token #ident #outer_impl
        #outer_where
        { inner: ::core::mem::MaybeUninit<#inner_ident #inner_ty> }

        #struct_token #inner_ident #inner_impl #inner_where
        #fields
        #semi_token

        impl #outer_impl ::core::ops::Deref for #ident #outer_ty #outer_where {
            type Target = #inner_ident #inner_ty;
            fn deref(&self) -> &Self::Target {
                unsafe { self.inner.assume_init_ref() }
            }
        }

        impl #outer_impl ::core::ops::DerefMut for #ident #outer_ty #outer_where {
            fn deref_mut(&mut self) -> &mut Self::Target {
                unsafe { self.inner.assume_init_mut() }
            }
        }

        impl #outer_impl Drop for #ident #outer_ty #outer_where {
            fn drop(&mut self) {
                unsafe { self.inner.assume_init_drop() }
            }
        }
    }
    .into()
}

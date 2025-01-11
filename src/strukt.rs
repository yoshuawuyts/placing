use proc_macro::TokenStream;

use quote::{format_ident, quote};
use syn::{spanned::Spanned, ItemStruct};

use crate::utils::{self, create_maybe_generics};

/// Process an impl block that carries the `#[placing]` notation
pub(crate) fn process_struct(item: ItemStruct) -> TokenStream {
    // We need all the impl components to later recreate it
    // and fill it with our own methods
    let ItemStruct {
        attrs: _,
        vis,
        struct_token,
        ident,
        generics,
        mut fields,
        semi_token,
    } = item;

    let inner_ident = format_ident!("Inner{}", ident);
    let maybe_generics = create_maybe_generics(&generics);
    let (maybe_impl, maybe_ty, maybe_where) = maybe_generics.split_for_impl();

    if let Err(err) = rewrite_placing_fields(&mut fields) {
        return err;
    }

    quote! {
        #vis #struct_token #ident #maybe_impl #maybe_where
        { inner: ::core::mem::MaybeUninit<#inner_ident #maybe_ty> }

        #struct_token #inner_ident #maybe_impl #maybe_where
        #fields
        #semi_token

        impl #maybe_impl ::core::ops::Deref for #ident #maybe_ty #maybe_where {
            type Target = #inner_ident #maybe_ty;
            fn deref(&self) -> &Self::Target {
                unsafe { self.inner.assume_init_ref() }
            }
        }

        impl #maybe_impl ::core::ops::DerefMut for #ident #maybe_ty #maybe_where {
            fn deref_mut(&mut self) -> &mut Self::Target {
                unsafe { self.inner.assume_init_mut() }
            }
        }

        impl #maybe_impl Drop for #ident #maybe_ty #maybe_where {
            fn drop(&mut self) {
                unsafe { self.inner.assume_init_drop() }
            }
        }
    }
    .into()
}

/// If a field has a `#[placing]` attribute, pass the generic in.
fn rewrite_placing_fields(fields: &mut syn::Fields) -> Result<(), TokenStream> {
    for field in fields {
        if utils::has_placing_attr(&field.attrs)? {
            utils::strip_placing_attr(&mut field.attrs);
            let syn::Type::Path(type_path) = &mut field.ty else {
                return Err(quote::quote_spanned! { field.ty.span() =>
                    compile_error!("[E0010, placing] invalid `#[placing]` target type"),
                }
                .into());
            };

            let segment = type_path
                .path
                .segments
                .last_mut()
                .expect("could not get last segment of type");
            match &mut segment.arguments {
                args @ syn::PathArguments::None => {
                    *args = syn::PathArguments::AngleBracketed(
                        syn::parse2(quote! { <EMPLACE> }).unwrap(),
                    );
                }
                syn::PathArguments::AngleBracketed(args) => {
                    args.args.push(syn::parse2(quote! { EMPLACE }).unwrap());
                }
                syn::PathArguments::Parenthesized(_) => {
                    unreachable!("who decided to put a Fn() -> function here?");
                }
            }
        }
    }
    Ok(())
}

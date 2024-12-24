use proc_macro::TokenStream;

use quote::quote;
use syn::{spanned::Spanned, ImplItem, ImplItemFn, ItemImpl, PathSegment};

use crate::utils::create_outer_generics;

/// Process an impl block that carries the `#[spati]` notation
pub(crate) fn process_impl(item: ItemImpl) -> TokenStream {
    if item.trait_.is_some() {
        return quote::quote! {
            compile_error!("`[spati, E0002] trait impls unsupported: macro `spati` can only be used on bare `impl {}` blocks");
        }.into();
    }
    // let span = item.span();

    // We need all the impl components to later recreate it
    // and fill it with our own methods
    let ItemImpl {
        attrs: _,
        defaultness,
        unsafety,
        impl_token,
        generics,
        trait_: _,
        self_ty,
        brace_token: _,
        items,
    } = item;

    // We only want to modify the methods, the rest of the items we're happy to
    // pass along as-is.
    let mut fn_items = vec![];
    let mut all_items = vec![];
    for item in items {
        match item {
            ImplItem::Fn(f) => fn_items.push(f),
            item => all_items.push(item),
        }
    }
    let fn_items = process_fns(fn_items);
    all_items.extend(fn_items);

    // Add the `const EMPLACE` generic to the list of generics
    let outer_generics = create_outer_generics(&generics);
    let (gen_impl, gen_ty, gen_where) = outer_generics.split_for_impl();

    // Add `const EMPLACE` to the list of generics on impl target type
    let mut self_ty = match *self_ty {
        syn::Type::Path(type_path) => type_path,
        _ => return quote::quote_spanned! {
            impl_token.span() => compile_error!("[E0003, spati] invalid impl target: `spati` doesn't work for impls on tuples, slices, or other non-path types"),
        }
        .into(),
    };

    let self_ident = &self_ty.path.segments.last_mut().unwrap().ident;
    *self_ty.path.segments.last_mut().unwrap() = create_segment(self_ident, &gen_ty);

    // All done now, send back our updated `impl` block
    quote! {
        #defaultness #unsafety #impl_token #gen_impl #self_ty #gen_where {
            #(#all_items)*
        }
    }
    .into()
}

fn create_segment(
    self_ident: &syn::Ident,
    ty_generics: &syn::TypeGenerics<'_>,
) -> syn::PathSegment {
    let item = quote! { #self_ident #ty_generics };
    let item: PathSegment = syn::parse2(item).unwrap();
    item
}

/// Process the
fn process_fns(fn_items: Vec<ImplItemFn>) -> Vec<ImplItem> {
    let mut output = vec![];
    for f in fn_items {
        output.push(ImplItem::Fn(f));
    }
    output
}

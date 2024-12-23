use proc_macro::TokenStream;

use quote::quote;
use syn::{ImplItem, ImplItemFn, ItemImpl};

/// Process an impl block that carries the `#[spati]` notation
pub(crate) fn process_impl(item: ItemImpl) -> TokenStream {
    if item.trait_.is_some() {
        return quote::quote! {
            compile_error!("`[spati, E0002] trait impls unsupported: macro `spati` can only be used on bare `impl {}` blocks");
        }.into();
    }

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

    // All done now, send back our updated `impl` block
    quote! {
        #defaultness #unsafety #impl_token #generics #self_ty {
            #(#all_items)*
        }
    }
    .into()
}

/// Process the
fn process_fns(fn_items: Vec<ImplItemFn>) -> Vec<ImplItem> {
    let mut output = vec![];
    for f in fn_items {
        output.push(ImplItem::Fn(f));
    }
    output
}

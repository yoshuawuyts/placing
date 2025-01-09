use proc_macro::TokenStream;

use quote::quote;
use syn::{spanned::Spanned, ImplItem, ImplItemFn, ItemImpl};

use crate::utils::{create_maybe_generics, has_super, set_path_generics, strip_super};

mod fn_kind;
mod moving_constructor;
mod placing_constructor;

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

    // Validate we're processing an impl we know how to handle
    let self_ty = match *self_ty {
        syn::Type::Path(type_path) => type_path,
        _ => return quote::quote_spanned! { impl_token.span() =>
            compile_error!("[E0003, spati] invalid impl target: `spati` doesn't work for impls on tuples, slices, or other non-path types"),
        }.into(),
    };
    let self_ident = &self_ty.path.segments.last().unwrap().ident.clone();

    // We need to generate three different impl blocks:
    // - one where EMPLACE is true
    // - one where EMPLACE is false
    // - one where EMPLACE is generic
    let self_ty_true = set_path_generics(&self_ty, &generics, syn::parse2(quote! {true}).unwrap());
    let self_ty_false =
        set_path_generics(&self_ty, &generics, syn::parse2(quote! {false}).unwrap());
    let self_ty_maybe =
        set_path_generics(&self_ty, &generics, syn::parse2(quote! {EMPLACE}).unwrap());

    // Create our final sets of generic params
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let maybe_generics = create_maybe_generics(&generics);
    let (impl_generics_maybe, _, where_clause_maybe) = maybe_generics.split_for_impl();

    // We only want to modify the methods, the rest of the items we're happy to
    // pass along as-is.
    let mut fn_items = vec![];
    let mut non_fn_items = vec![];
    for item in items {
        match item {
            ImplItem::Fn(f) => fn_items.push(f),
            item => non_fn_items.push(item),
        }
    }
    let fns = match rewrite_fns(fn_items, &self_ident) {
        Ok(fn_items) => fn_items,
        Err(token_stream) => return token_stream,
    };

    let ImplFns {
        statics,
        methods,
        emplacing_constructors,
        non_emplacing_constructors,
    } = fns;

    // All done now, send back our updated `impl` block
    quote! {
        #defaultness #unsafety #impl_token #impl_generics #self_ty_false #where_clause {
            #(#non_emplacing_constructors)*
            #(#non_fn_items)*
            #(#statics)*
        }
        #defaultness #unsafety #impl_token #impl_generics #self_ty_true #where_clause {
            #(#emplacing_constructors)*
        }
        #defaultness #unsafety #impl_token #impl_generics_maybe #self_ty_maybe #where_clause_maybe {
            #(#methods)*
        }
    }
    .into()
}

/// The output of `rewrite_fns`
#[derive(Default)]
struct ImplFns {
    statics: Vec<ImplItem>,
    methods: Vec<ImplItem>,
    emplacing_constructors: Vec<ImplItem>,
    non_emplacing_constructors: Vec<ImplItem>,
}

/// Process thef functions one by one
fn rewrite_fns(fn_items: Vec<ImplItemFn>, self_ident: &syn::Ident) -> Result<ImplFns, TokenStream> {
    let mut output = ImplFns::default();
    for mut f in fn_items {
        // Process and identify the function
        let fn_kind = fn_kind::FunctionKind::from_fn(&f.sig, self_ident);
        let has_super = has_super(&f.attrs)?;
        strip_super(&mut f.attrs);

        // Validate the function bodies and rewrite them where needed.
        match (&fn_kind, has_super) {
            (fn_kind::FunctionKind::Static, false) => {
                output.statics.push(f.into());
            }
            (fn_kind::FunctionKind::Method, false) => {
                output.methods.push(f.into());
            }
            (fn_kind::FunctionKind::Static, true) => {
                return Err(quote::quote_spanned! { f.sig.span() =>
                    compile_error!("[E0007, spati] invalid super target: the #[super] attribute cannot be applied to static functions"),
                }.into());
            }
            (fn_kind::FunctionKind::Method, true) => {
                return Err(quote::quote_spanned! { f.sig.span() =>
                    compile_error!("[E0007, spati] invalid super target: the #[super] attribute cannot be applied to static functions"),
                }.into());
            }
            (fn_kind::FunctionKind::Constructor(_heap_ty), false) => {
                moving_constructor::rewrite_moving_constructor(&mut output, f, self_ident)?;
            }
            (fn_kind::FunctionKind::Constructor(_heap_ty), true) => {
                placing_constructor::rewrite_super_constructor(&mut output, f.clone())?;
                // TODO: re-enable me
                // moving_constructor::rewrite_moving_constructor(&mut output, f, self_ident)?;
            }
            (fn_kind::FunctionKind::Builder(_heap_ty), true) => {
                todo!("builders and transforms not yet supported")
            }
            (fn_kind::FunctionKind::Builder(_heap_ty), false) => {
                todo!("builders and transforms not yet supported")
            }
        }
    }
    Ok(output)
}

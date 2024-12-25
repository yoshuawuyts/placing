use std::{any::type_name, process::Output};

use proc_macro::TokenStream;

use quote::quote;
use syn::{
    spanned::Spanned, Attribute, FnArg, ImplItem, ImplItemFn, ItemImpl, PatType, Path, PathSegment,
    Receiver, ReturnType,
};

use crate::utils::create_outer_generics;

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

    // Add the `const EMPLACE` generic to the list of generics
    let outer_generics = create_outer_generics(&generics);
    let (gen_impl, gen_ty, gen_where) = outer_generics.split_for_impl();

    // Add `const EMPLAVCE` to the list of generics on impl target type
    let mut self_ty = match *self_ty {
        syn::Type::Path(type_path) => type_path,
        _ => return quote::quote_spanned! { impl_token.span() =>
            compile_error!("[E0003, spati] invalid impl target: `spati` doesn't work for impls on tuples, slices, or other non-path types"),
        }.into(),
    };
    let self_ident = &self_ty.path.segments.last().unwrap().ident.clone();
    *self_ty.path.segments.last_mut().unwrap() = update_target_ident(self_ident, &gen_ty);

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
    let fn_items = match rewrite_fns(fn_items, &self_ident) {
        Ok(fn_items) => fn_items,
        Err(token_stream) => return token_stream,
    };
    all_items.extend(fn_items);

    // All done now, send back our updated `impl` block
    quote! {
        #defaultness #unsafety #impl_token #gen_impl #self_ty #gen_where {
            #(#all_items)*
        }
    }
    .into()
}

/// Update the target type of the impl block with the right generics
fn update_target_ident(
    self_ident: &syn::Ident,
    ty_generics: &syn::TypeGenerics<'_>,
) -> syn::PathSegment {
    let item = quote! { #self_ident #ty_generics };
    let item: PathSegment = syn::parse2(item).unwrap();
    item
}

/// Process the
fn rewrite_fns(
    fn_items: Vec<ImplItemFn>,
    self_ident: &syn::Ident,
) -> Result<Vec<ImplItem>, TokenStream> {
    let mut output = vec![];
    for f in fn_items {
        let fn_kind = FunctionKind::from_fn(&f, self_ident);

        let ImplItemFn {
            mut attrs,
            vis,
            defaultness,
            sig,
            block,
        } = f;

        // Check whether `super` is present and then remove it from the list
        // errors if the attribute is present but malformed
        let has_super = has_super(&attrs)?;
        strip_super(&mut attrs);

        let statements = block.stmts;
        let f = syn::parse2(quote! {
            #(#attrs)*
            #vis #defaultness #sig {
                #(#statements)*
            }
        })
        .unwrap();
        output.push(ImplItem::Fn(f));
    }
    Ok(output)
}

/// What kind of function are we operating on?
enum FunctionKind {
    /// A static method with no self-ty
    Static,
    /// A static constructor with a return type of `Self`
    Constructor,
    /// A method with a self-ty
    Method,
}

impl FunctionKind {
    fn from_fn(f: &ImplItemFn, self_ident: &syn::Ident) -> Self {
        // Check whether `-> Self` or equivalent
        if let ReturnType::Type(_, ty) = &f.sig.output {
            if let syn::Type::Path(type_path) = &**ty {
                let self_ident = self_ident.to_string();
                let ty_path = path_to_string(&type_path.path);
                if ty_path == "Self" || ty_path == self_ident {
                    return Self::Constructor;
                }
            }
        }

        // check the input arguments for whether it has `self` or `self: Pattern {}`
        match f.sig.inputs.first() {
            Some(FnArg::Receiver(Receiver { ty, .. })) | Some(FnArg::Typed(PatType { ty, .. })) => {
                match &**ty {
                    syn::Type::Path(type_path) => match path_to_string(&type_path.path).as_str() {
                        "self" => Self::Method,
                        _ => panic!("good job, you're a human fuzzer"),
                    },
                    _ => panic!("good job, you're a human fuzzer"),
                }
            }
            None => Self::Static,
        }
    }
}

fn has_super(attrs: &[Attribute]) -> Result<bool, TokenStream> {
    for attr in attrs.iter() {
        if path_to_string(attr.path()) != "super" {
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

fn path_to_string(path: &Path) -> String {
    match path.get_ident() {
        Some(ident) => ident.to_string(),
        None => "".to_string(),
    }
}

fn strip_super(attrs: &mut Vec<Attribute>) {
    attrs.retain(|attr| path_to_string(attr.path()) != "super")
}

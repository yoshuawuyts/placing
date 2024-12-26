use proc_macro::TokenStream;

use quote::quote;
use syn::{
    spanned::Spanned, Attribute, Block, FnArg, ImplItem, ImplItemFn, ItemImpl, Path, PathSegment,
    Receiver, ReturnType, Signature, Stmt,
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
        let ImplItemFn {
            mut attrs,
            vis,
            defaultness,
            sig,
            mut block,
        } = f;

        // Process and identify the function
        let fn_kind = dbg!(FunctionKind::from_fn(&sig, self_ident));
        let has_super = dbg!(has_super(&attrs)?);
        strip_super(&mut attrs);

        // Validate the function bodies and rewrite them where needed.
        match (fn_kind, has_super) {
            (FunctionKind::Static, false) | (FunctionKind::Method, false) => {}
            (FunctionKind::Static, true) => {
                return Err(quote::quote_spanned! { sig.span() =>
                    compile_error!("[E0007, spati] invalid super target: the #[super] attribute cannot be applied to static functions"),
                }.into());
            }
            (FunctionKind::Method, true) => {
                return Err(quote::quote_spanned! { sig.span() =>
                    compile_error!("[E0007, spati] invalid super target: the #[super] attribute cannot be applied to static functions"),
                }.into());
            }
            (FunctionKind::Constructor, false) => todo!(),

            (FunctionKind::Constructor, true) => rewrite_super_constructor(&mut block, self_ident)?,
            (FunctionKind::Builder, true) => todo!(),
            (FunctionKind::Builder, false) => todo!(),
        }

        let f = syn::parse2(quote! {
            #(#attrs)*
            #vis #defaultness #sig
            #block
        })
        .unwrap();
        output.push(ImplItem::Fn(f));
    }
    Ok(output)
}

/// What kind of function are we operating on?
#[derive(Debug)]
enum FunctionKind {
    /// A static method with no self-ty
    Static,
    /// A static constructor with a return type of `Self`
    Constructor,
    /// A method with a self-ty
    Method,
    /// A method with a self-ty that returns type `Self`
    Builder,
}

impl FunctionKind {
    fn from_fn(sig: &Signature, self_ident: &syn::Ident) -> Self {
        // If the function `-> Self` or equivalent we're working with a
        // constructor
        if let ReturnType::Type(_, ty) = dbg!(&sig.output) {
            if let syn::Type::Path(type_path) = &**ty {
                let ty_path = path_to_string(&type_path.path);
                let self_ident = self_ident.to_string();
                if ty_path == "Self" || ty_path == self_ident {
                    match sig.inputs.first() {
                        Some(FnArg::Receiver(Receiver { .. })) => return Self::Builder,
                        _ => return Self::Constructor,
                    };
                }
            }
        }

        // If our function takes `self` or `self: Pattern {}`, we're working with a
        // method. Otherwise it's a static function.
        match sig.inputs.first() {
            Some(FnArg::Receiver(Receiver { .. })) => Self::Method,
            _ => Self::Static,
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

/// Validate that the `super` attribute is correctly applied
fn rewrite_super_constructor(block: &mut Block, ident: &syn::Ident) -> Result<(), TokenStream> {
    match block.stmts.last_mut() {
        Some(syn::Stmt::Expr(expr, _)) => {
            match expr {
                syn::Expr::Struct(strukt) => {
                    // TODO: validate #strukt has the right name
                    // TODO: rewrite strukt's name
                    *strukt = syn::parse2(quote! {
                        #ident {
                            inner: #strukt
                        }
                    }).unwrap();
                    Ok(())
                }
                expr => Err(quote::quote_spanned! { expr.span() =>
                    compile_error!("[E0006, spati] invalid constructor body: functions marked `#[super]` have to end with a struct expression"),
                }.into()),
            }
        }
        Some(stmt) => Err(quote::quote_spanned! { stmt.span() =>
            compile_error!("[E0006, spati] invalid constructor body: functions marked `#[super]` have to end with a struct expression"),
        }.into()),
        None => Err(quote::quote_spanned! { block.span() =>
            compile_error!("[E0005, spati] empty constructor body: functions marked `#[super]` cannot be empty"),
        }.into()),
    }
}

/// Rewrite a non-`#[super]` statement to create the inner type instead
fn rewrite_non_super_constructor(statement: &Stmt, target: &syn::Ident) -> Stmt {
    syn::parse2(quote! {
        #target {
            inner: #statement
        }
    })
    .unwrap()
}

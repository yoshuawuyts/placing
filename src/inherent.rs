use proc_macro::TokenStream;

use quote::{format_ident, quote};
use syn::{
    spanned::Spanned, token::Impl, Attribute, Block, FnArg, ImplItem, ImplItemFn, ItemImpl, Path,
    Receiver, ReturnType, Signature,
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

    // Validate we're processing an impl we know how to handle
    let self_ty = match *self_ty {
        syn::Type::Path(type_path) => type_path,
        _ => return quote::quote_spanned! { impl_token.span() =>
            compile_error!("[E0003, spati] invalid impl target: `spati` doesn't work for impls on tuples, slices, or other non-path types"),
        }.into(),
    };
    let self_ident = &self_ty.path.segments.last().unwrap().ident.clone();

    // We need to generate three different impl blocks:
    // - one where EMPLACE is generic
    // - one where EMPLACE is true
    // - one where EMPLACE is false
    let self_ty_true = set_path_generics(&self_ty, &generics, syn::parse2(quote! {true}).unwrap());
    let self_ty_false =
        set_path_generics(&self_ty, &generics, syn::parse2(quote! {false}).unwrap());
    let self_ty_generic =
        set_path_generics(&self_ty, &generics, syn::parse2(quote! {EMPLACE}).unwrap());

    // Create our final sets of generic params
    let (gen_impl, _, gen_where) = generics.split_for_impl();
    let (gen_impl_true, self_ty_true, gen_where_true) =
        (gen_impl.clone(), self_ty_true, gen_where.clone());
    let (gen_impl_false, self_ty_false, gen_where_false) =
        (gen_impl.clone(), self_ty_false, gen_where.clone());
    let outer_generics = create_outer_generics(&generics);
    let (gen_impl, _, gen_where) = outer_generics.split_for_impl();

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
    let ImplFns {
        statics,
        methods,
        emplacing_constructors,
        non_emplacing_constructors,
    } = match rewrite_fns(fn_items, &self_ident) {
        Ok(fn_items) => fn_items,
        Err(token_stream) => return token_stream,
    };

    // All done now, send back our updated `impl` block
    quote! {
        #defaultness #unsafety #impl_token #gen_impl_false #self_ty_false #gen_where_false {
            #(#non_emplacing_constructors)*
            #(#non_fn_items)*
            #(#statics)*
        }
        #defaultness #unsafety #impl_token #gen_impl_true #self_ty_true #gen_where_true {
            #(#emplacing_constructors)*
        }
        #defaultness #unsafety #impl_token #gen_impl #self_ty_generic #gen_where {
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
        let fn_kind = FunctionKind::from_fn(&f.sig, self_ident);
        let has_super = has_super(&f.attrs)?;
        strip_super(&mut f.attrs);

        // Validate the function bodies and rewrite them where needed.
        match (&fn_kind, has_super) {
            (FunctionKind::Static, false) => {
                output.statics.push(f.into());
            }
            (FunctionKind::Method, false) => {
                output.methods.push(f.into());
            }
            (FunctionKind::Static, true) => {
                return Err(quote::quote_spanned! { f.sig.span() =>
                    compile_error!("[E0007, spati] invalid super target: the #[super] attribute cannot be applied to static functions"),
                }.into());
            }
            (FunctionKind::Method, true) => {
                return Err(quote::quote_spanned! { f.sig.span() =>
                    compile_error!("[E0007, spati] invalid super target: the #[super] attribute cannot be applied to static functions"),
                }.into());
            }
            (FunctionKind::Constructor, false) => {
                rewrite_non_super_constructor(&mut f.block, self_ident)?;
                output.non_emplacing_constructors.push(f.into());
            }
            (FunctionKind::Constructor, true) => {
                rewrite_super_constructor(&mut output, f, self_ident)?;
            }
            (FunctionKind::Builder, true) => todo!("builders and transforms not yet supported"),
            (FunctionKind::Builder, false) => todo!("builders and transforms not yet supported"),
        }
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
        if let ReturnType::Type(_, ty) = &sig.output {
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

/// Rewrite a `#[super]` statement to create the inner type instead
fn rewrite_super_constructor(
    output: &mut ImplFns,
    mut f: ImplItemFn,
    ident: &syn::Ident,
) -> Result<(), TokenStream> {
    let inner_ty_ident = format_ident!("Inner{}", ident);
    let fn_ident = f.sig.ident.clone();

    // Create the new uninit constructor
    let syn::ImplItemFn {
        attrs,
        vis,
        defaultness,
        sig,
        block,
    } = &f;
    let Signature {
        constness: _,
        asyncness: _,
        unsafety: _,
        abi: _,
        fn_token,
        ident: _,
        generics: _,
        paren_token: _,
        inputs: _,
        variadic: _,
        output: _,
    } = sig;
    let uninit_ident = format_ident!("spati_uninit_{}", fn_ident);
    let uninit: syn::ImplItemFn = syn::parse2(quote! {
        #(#attrs)*
        #vis #fn_token #uninit_ident() -> Self
        {
            Self{ inner: ::core::mem::MaybeUninit::uninit() }
        }
    })
    .unwrap();
    output.emplacing_constructors.push(uninit.into());

    // Rewrite the existing constructors body to emplace
    match f.block.stmts.last_mut() {
        Some(syn::Stmt::Expr(expr, _)) => {
            match expr {
                syn::Expr::Struct(strukt) => {

                    let assignments = strukt.fields.iter().map(|field| {
                        let key = &field.member; 
                        let expr = &field.expr;
                        syn::parse2(quote! { {{
                            let _this = self.inner.as_mut_ptr();
                            unsafe { (&raw mut (*_this).#key).write(#expr) };
                        }} }).unwrap()
                    }).collect::<Vec<syn::ExprBlock>>();
                    *strukt = syn::parse2(quote! {
                        let _this = self.inner.as_mut_ptr();

                        #(#assignments)*;
                    }).unwrap();
                }
                expr => return Err(quote::quote_spanned! { expr.span() =>
                    compile_error!("[E0006, spati] invalid constructor body: functions marked `#[super]` have to end with a struct expression"),
                }.into()),
            }
        }
        Some(stmt) => return Err(quote::quote_spanned! { stmt.span() =>
            compile_error!("[E0006, spati] invalid constructor body: functions marked `#[super]` have to end with a struct expression"),
        }.into()),
        None => return Err(quote::quote_spanned! { f.block.span() =>
            compile_error!("[E0005, spati] empty constructor body: functions marked `#[super]` cannot be empty"),
        }.into()),

    };
    // TODO: Rewrite the existing constructors signature to emplace
    f.sig.ident = format_ident!("spati_init_{}", fn_ident);
    f.sig.output = ReturnType::Default;
    let inputs = f.sig.inputs.clone();
    f.sig.inputs.clear();
    f.sig
        .inputs
        .push(FnArg::Receiver(syn::parse2(quote! { &mut self }).unwrap()));
    f.sig.inputs.extend(inputs);
    output.emplacing_constructors.push(f.into());
    Ok(())
}

/// Rewrite a non-`#[super]` statement to create the inner type instead
fn rewrite_non_super_constructor(block: &mut Block, ident: &syn::Ident) -> Result<(), TokenStream> {
    let inner_ident = format_ident!("Inner{}", ident);
    match block.stmts.last_mut() {
        Some(syn::Stmt::Expr(expr, _)) => {
            match expr {
                syn::Expr::Struct(strukt) => {
                    let fields = strukt.fields.clone();
                    // TODO: validate #strukt has the right name
                    // TODO: rewrite strukt's name
                    *strukt = syn::parse2(quote! {
                        #ident {
                            inner: ::core::mem::MaybeUninit::new(#inner_ident { #fields })
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

fn set_path_generics(
    path: &syn::TypePath,
    base: &syn::Generics,
    param: syn::GenericArgument,
) -> syn::TypePath {
    let mut path = path.clone();
    let segment = path.path.segments.last_mut().unwrap();
    let ident = &segment.ident;
    let params = base.params.iter().map(|param| -> syn::GenericArgument {
        match param {
            syn::GenericParam::Lifetime(lifetime_param) => {
                let param = &lifetime_param.lifetime;
                syn::parse2(quote! {#param}).unwrap()
            }
            syn::GenericParam::Type(type_param) => {
                let param = &type_param.ident;
                syn::parse2(quote! {#param}).unwrap()
            }
            syn::GenericParam::Const(const_param) => {
                let param = &const_param.ident;
                syn::parse2(quote! {#param}).unwrap()
            }
        }
    });
    *segment = syn::parse2(quote! { #ident <#(#params,)* #param> }).unwrap();
    path
}

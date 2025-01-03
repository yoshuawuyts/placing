use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{spanned::Spanned, ImplItemFn};

use super::ImplFns;

/// Rewrite a non-`#[super]` statement to create the inner type instead
pub(crate) fn rewrite_moving_constructor(
    output: &mut ImplFns,
    mut f: ImplItemFn,
    ident: &syn::Ident,
) -> Result<(), TokenStream> {
    let inner_ident = format_ident!("Inner{}", ident);
    match f.block.stmts.last_mut() {
        Some(syn::Stmt::Expr(expr, _)) => {
            match expr {
                syn::Expr::Struct(strukt) => {
                    let fields = strukt.fields.clone();
                    *strukt = syn::parse2(quote! {
                        #ident {
                            inner: ::core::mem::MaybeUninit::new(#inner_ident { #fields })
                        }
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
    output.non_emplacing_constructors.push(f.into());
    Ok(())
}

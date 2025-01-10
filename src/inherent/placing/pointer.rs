use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

use crate::utils::expr_path_ident;

/// Convert `Box::new(Self { .. })` to writes into a `Box<MaybeUninit<Self>>`
pub(crate) fn pointer_new_init(call: &mut syn::ExprCall) -> Result<syn::Expr, TokenStream> {
    let syn::Expr::Path(path) = &*call.func else {
        return Err(quote::quote_spanned! { call.span() =>
            compile_error!("[E0006, spati] invalid constructor body: functions marked `#[super]` can only end with a fixed set of expressions"),
        }.into());
    };

    match expr_path_ident(path).as_str() {
        "Box :: new" => {}
        _ => {
            return Err(quote::quote_spanned! { call.span() =>
                compile_error!("[E0008, spati] invalid pointer constructor"),
            }
            .into())
        }
    }

    let strukt = match call.args.first() {
        Some(syn::Expr::Struct(expr)) => expr,
        _ => {
            return Err(quote::quote_spanned! { call.span() =>
                compile_error!("[E0008, spati] invalid pointer constructor`"),
            }
            .into())
        }
    };

    let assignments = strukt
        .fields
        .iter()
        .map(|field| {
            let key = &field.member;
            let expr = &field.expr;
            syn::parse2(quote! {{
                unsafe { (&raw mut (*_this).#key).write(#expr) };
            }})
            .unwrap()
        })
        .collect::<Vec<syn::Block>>();

    Ok(syn::parse2(quote! {{
        let _this = self.inner.as_mut_ptr();
        #(#assignments)*
    }})
    .unwrap())
}

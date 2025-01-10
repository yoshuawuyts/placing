use quote::quote;
/// Convert `Self { .. }` to writes into a `MaybeUninit<Self>`
pub(crate) fn inline_new_init(strukt: &mut syn::ExprStruct) -> syn::Expr {
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

    syn::parse2(quote! {{
        let _this = self.inner.as_mut_ptr();
        #(#assignments)*
    }})
    .unwrap()
}

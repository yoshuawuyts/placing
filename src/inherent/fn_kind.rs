use crate::utils::path_ident;
use syn::{FnArg, Receiver, ReturnType, Signature};

/// What kind of function are we operating on?
#[derive(Debug)]
pub(crate) enum FunctionKind {
    /// A static method with no self-ty
    Static,
    /// A static constructor with a return type of `Self`
    Constructor(HeapTy),
    /// A method with a self-ty
    Method,
    /// A method with a self-ty that returns type `Self`
    Builder(HeapTy),
}

/// Is our `Self`-ty on the heap?
#[derive(Debug)]
pub(crate) enum HeapTy {
    /// `-> Self`
    None,
    /// `-> Box<Self>`
    Box,
}

impl FunctionKind {
    pub(crate) fn from_fn(sig: &Signature, self_ident: &syn::Ident) -> Self {
        // If the function contains `-> Self` or equivalent we're working with a
        // constructor
        if let ReturnType::Type(_, ty) = &sig.output {
            if let syn::Type::Path(type_path) = &**ty {
                let ty_path = path_ident(&type_path.path);
                if ty_path == "Self" || ty_path == self_ident.to_string() {
                    match sig.inputs.first() {
                        Some(FnArg::Receiver(Receiver { .. })) => {
                            return Self::Builder(HeapTy::None)
                        }
                        _ => return Self::Constructor(HeapTy::None),
                    };
                }

                if ty_path == "Box < Self >" {
                    match sig.inputs.first() {
                        Some(FnArg::Receiver(Receiver { .. })) => {
                            return Self::Builder(HeapTy::Box)
                        }
                        _ => return Self::Constructor(HeapTy::Box),
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

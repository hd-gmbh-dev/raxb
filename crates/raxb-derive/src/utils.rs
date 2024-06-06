use std::str::FromStr;

use quote::quote;

use crate::container::{BuiltInConstType, BuiltInType, Generic, StructField};

pub fn create_ident(f: &StructField) -> proc_macro2::TokenStream {
    let ident = f.original.ident.as_ref().unwrap();
    let ty = &f.original.ty;
    match f.generic {
        Generic::Vec(v) => quote! {
            let mut #ident = Vec::<#v>::new();
        },
        Generic::Opt(opt) => quote! {
            let mut #ident = Option::<#opt>::None;
        },
        Generic::None => quote! {
            let mut #ident = Option::<#ty>::None;
        },
    }
}

pub fn get_built_in_type(ty: &syn::Type) -> BuiltInType {
    if let syn::Type::Path(p) = ty {
        if let Some(ty_ident) = p.path.get_ident() {
            return BuiltInType::from_str(&format!("{ty_ident}")).unwrap_or_default();
        }
    }
    BuiltInType::Unknown
}

pub fn get_built_in_const_type(ty: &syn::Type) -> BuiltInConstType {
    if let syn::Type::Path(p) = ty {
        if let Some(ty_ident) = p.path.get_ident() {
            return BuiltInConstType::from_str(&format!("{ty_ident}")).unwrap_or_default();
        }
    }
    BuiltInConstType::Unknown
}

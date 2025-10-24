use std::str::FromStr;

use quote::quote;

use crate::container::{BuiltInConstType, BuiltInType, Container, Generic, NsValue, StructField};

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

pub fn create_tns_impl(container: &Container) -> proc_macro2::TokenStream {
    if let Some((_, ns_val)) = container.tns.as_ref() {
        let ns_token = match ns_val {
            NsValue::LitByte(lit) => quote! { #lit },
            NsValue::ExprPath(path) => quote! { #path },
        };
        quote! {
            fn target_ns() -> Option<_raxb::ty::XmlTargetNs> {
                Some(#ns_token)
            }
        }
    } else {
        quote! {}
    }
}

pub fn create_root_impl(container: &Container) -> proc_macro2::TokenStream {
    if let Some(root) = container.root.as_ref() {
        quote! {
            fn root() -> Option<_raxb::ty::XmlTag> {
                Some(#root)
            }
        }
    } else {
        quote! {}
    }
}

pub fn trace(
    #[allow(unused_variables)] content: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    #[cfg(feature = "trace")]
    quote! {
        {
            #[allow(unused_extern_crates)]
            extern crate raxb as _raxb;

            use _raxb::tracing::*;

            #content
        }
    }
    #[cfg(not(feature = "trace"))]
    quote! {}
}

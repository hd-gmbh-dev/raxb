use quote::quote;

use crate::container::{Generic, StructField};

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

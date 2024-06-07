use quote::quote;
use syn::DeriveInput;

use crate::container::{self, Container};

mod attrs;
mod child;
mod text;
mod ty_enum;
mod ty_struct;

pub fn xml_serialize_impl_block(input: DeriveInput) -> proc_macro2::TokenStream {
    let container = Container::from_ast(&input, container::Derive::Deserialize);
    // eprintln!("validate container");
    container.validate();
    if container.is_enum() {
        // eprintln!("run ty_enum::impl_block");
        ty_enum::impl_block(container)
    } else {
        let is_simple_type = container
            .struct_fields
            .iter()
            .all(|sf| sf.original.ident.is_none());
        if is_simple_type {
            // eprintln!("run ty_simple::impl_block");
            // ty_simple::impl_block(container)
            quote! {}
        } else {
            // eprintln!("run ty_struct::impl_block");
            ty_struct::impl_block(container)
        }
    }
}

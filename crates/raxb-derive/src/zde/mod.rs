use crate::container::{self, Container};
use syn::DeriveInput;

// mod attrs;
// mod child;
// mod text;
// mod ty_enum;
// mod ty_simple;
mod ty_struct;

pub fn xml_borrow_impl_block(input: DeriveInput) -> proc_macro2::TokenStream {
    let container = Container::from_ast(&input, container::Derive::Deserialize);
    // eprintln!("validate container");
    container.validate();
    ty_struct::impl_block(container)
}

#![doc = include_str!("../README.md")]
/*
#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    variant_size_differences
)]
*/

mod container;
mod de;
mod ser;
mod symbol;
mod utils;
mod zde;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

use de::xml_deserialize_impl_block;
use ser::xml_serialize_impl_block;
use zde::xml_borrow_impl_block;

#[proc_macro_derive(XmlDeserialize, attributes(raxb, xml))]
pub fn derive_xml_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    xml_deserialize_impl_block(input).into()
}

#[proc_macro_derive(XmlBorrow, attributes(raxb, xml))]
pub fn derive_xml_borrow(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    xml_borrow_impl_block(input).into()
}

#[proc_macro_derive(XmlSerialize, attributes(raxb, xml))]
pub fn derive_xml_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    xml_serialize_impl_block(input).into()
}

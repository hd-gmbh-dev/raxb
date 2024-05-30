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
mod symbol;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

use de::xml_deserialize_impl_block;

#[proc_macro_derive(XmlDeserialize, attributes(raxb))]
pub fn derive_xml_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    xml_deserialize_impl_block(input).into()
}
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

pub use raxb_derive::XmlDeserialize;

pub mod de;
pub mod ser;

pub use quick_xml;

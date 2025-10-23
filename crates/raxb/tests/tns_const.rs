#![allow(dead_code, unused_imports)]

use raxb::{value::ConstStr, XmlDeserialize, XmlSerialize};

/// This test demonstrates that the `tns` attribute can accept a `const`
/// identifier (or any expression that evaluates to `&'static [u8]`) as its
/// second tuple element.
///
/// The constant is defined at the module level, but it could also be placed
/// inside an impl block. The generated code should treat it exactly like a
/// byte‑string literal.
pub static EX_NS: &[u8] = b"https://my.example.org/";

#[derive(Debug, XmlDeserialize, XmlSerialize)]
#[xml(root = b"Envelope")]
#[xml(tns(b"ex", EX_NS))]
pub struct Example {
    #[xml(
        default,
        ns = b"xmlns",
        name = b"ex",
        ty = "attr",
        value = "https://my.example.org/"
    )]
    _xmlns: ConstStr,
    #[xml(ns = b"ex", name = b"header", ty = "child")]
    pub header: Header,
}

#[derive(Debug, XmlDeserialize, XmlSerialize)]
pub struct Header {
    #[xml(ty = "text")]
    pub content: String,
}

#[test]
fn test_const_tns_serde() -> anyhow::Result<()> {
    // Serialize
    let xml = raxb::ser::to_string(&Example {
        header: Header {
            content: "BASE_64_ENCODED_XML".to_string(),
        },
        _xmlns: Default::default(),
    })?;
    assert_eq!(
        xml,
        r#"<ex:Envelope xmlns:ex="https://my.example.org/"><ex:header>BASE_64_ENCODED_XML</ex:header></ex:Envelope>"#
    );

    // Deserialize
    let de: Example = raxb::de::from_str(&xml)?;
    assert_eq!(de.header.content, "BASE_64_ENCODED_XML");
    Ok(())
}

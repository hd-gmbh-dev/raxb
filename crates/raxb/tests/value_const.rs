#![allow(dead_code, unused_imports)]

use raxb::{value::ConstStr, XmlDeserialize, XmlSerialize};

/// This test demonstrates that the `value` attribute can accept a `const`
/// identifier (or any expression that evaluates to `&'static str`) as its
/// value.
///
/// The constant is defined at the module level, but it could also be placed
/// inside an impl block. The generated code should treat it exactly like a
/// string literal.
pub const DEFAULT_NS: &str = "https://my.example.org/";

/// Another const for testing different values
pub const SCHEMA_LOCATION: &str = "https://myexample.org/examplev100 example.xsd";

#[derive(Default, Debug, XmlSerialize, XmlDeserialize)]
#[xml(root = b"a")]
pub struct A {
    #[xml(name = b"product", ty = "attr", value = "H & D")]
    pub product: ConstStr,
    #[xml(
        ns = b"xmlns",
        name = b"xsi",
        ty = "attr",
        value = DEFAULT_NS
    )]
    pub xmlns_xsi: ConstStr,
    #[xml(
        ns = b"xsi",
        name = b"schemaLocation",
        ty = "attr",
        value = SCHEMA_LOCATION
    )]
    pub schema_location: ConstStr,
}

#[test]
fn test_const_value_serde() -> anyhow::Result<()> {
    let test_xml = r#"<a product="H &amp; D" xmlns:xsi="https://my.example.org/" xsi:schemaLocation="https://myexample.org/examplev100 example.xsd"/>"#;
    let a1 = A::default();
    let s = raxb::ser::to_string(&a1)?;
    assert_eq!(s, test_xml);
    let a2: A = raxb::de::from_str(test_xml)?;
    assert_eq!(a2.product.as_ref(), "H & D");
    assert_eq!(a2.xmlns_xsi.as_ref(), "https://my.example.org/");
    assert_eq!(
        a2.schema_location.as_ref(),
        "https://myexample.org/examplev100 example.xsd"
    );
    Ok(())
}

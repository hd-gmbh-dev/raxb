use raxb::{value::ConstStr, XmlDeserialize, XmlSerialize};

// Private constants for attribute values (test scope only)
const DEFAULT_NS: &str = "https://my.example.org/";
const SCHEMA_LOCATION: &str = "https://myexample.org/examplev100 example.xsd";

/// Test struct demonstrating const value support for the `value` attribute.
/// Mixes literal string values with const identifiers to verify both work.
#[derive(Default, Debug, XmlSerialize, XmlDeserialize)]
#[xml(root = b"a")]
struct A {
    // Literal string value (existing behavior)
    #[xml(name = b"product", ty = "attr", value = "H & D")]
    product: ConstStr,
    // Const identifier value (new behavior)
    #[xml(ns = b"xmlns", name = b"xsi", ty = "attr", value = DEFAULT_NS)]
    xmlns_xsi: ConstStr,
    // Another const identifier
    #[xml(ns = b"xsi", name = b"schemaLocation", ty = "attr", value = SCHEMA_LOCATION)]
    schema_location: ConstStr,
}

/// Tests that const identifiers work for the `value` attribute.
/// The const values are embedded at compile time during serialization.
#[test]
fn test_const_value_serde() -> anyhow::Result<()> {
    // Serialize - const values should be embedded in output XML
    let expected_xml = r#"<a product="H &amp; D" xmlns:xsi="https://my.example.org/" xsi:schemaLocation="https://myexample.org/examplev100 example.xsd"/>"#;
    let a1 = A::default();
    let s = raxb::ser::to_string(&a1)?;
    assert_eq!(s, expected_xml);

    // Deserialize - values are read from XML, const definitions are not used
    let a2: A = raxb::de::from_str(expected_xml)?;
    assert_eq!(a2.product.as_ref(), "H & D");
    assert_eq!(a2.xmlns_xsi.as_ref(), "https://my.example.org/");
    assert_eq!(
        a2.schema_location.as_ref(),
        "https://myexample.org/examplev100 example.xsd"
    );

    // Deserialize - modify expected xml to assert different schema location
    let expected_xml = r#"<a product="H &amp; D" xmlns:xsi="https://my.example.org/" xsi:schemaLocation="https://myexample.org/examplev101 example.xsd"/>"#;
    let a2: A = raxb::de::from_str(expected_xml)?;
    assert_eq!(a2.product.as_ref(), "H & D");
    assert_eq!(a2.xmlns_xsi.as_ref(), "https://my.example.org/");
    assert_eq!(
        a2.schema_location.as_ref(),
        "https://myexample.org/examplev101 example.xsd"
    );

    Ok(())
}

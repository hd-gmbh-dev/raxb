use raxb::{value::ConstStr, XmlDeserialize, XmlSerialize};

// Private constant for namespace (test scope only)
const EX_NS: &[u8] = b"https://my.example.org/";

/// Test struct demonstrating const value support for the `tns` attribute.
/// The const value is used as the namespace URI in the generated XML.
#[derive(Debug, XmlDeserialize, XmlSerialize)]
#[xml(root = b"Envelope")]
#[xml(tns(b"ex", EX_NS))]
struct Example {
    // Default namespace declaration
    #[xml(
        default,
        ns = b"xmlns",
        name = b"ex",
        ty = "attr",
        value = "https://my.example.org/"
    )]
    _xmlns: ConstStr,
    // Namespaced child element
    #[xml(ns = b"ex", name = b"header", ty = "child")]
    header: Header,
}

/// Simple struct for nested content
#[derive(Debug, XmlDeserialize, XmlSerialize)]
struct Header {
    #[xml(ty = "text")]
    content: String,
}

/// Tests that const identifiers work for the `tns` attribute.
/// The const namespace URI is embedded at compile time during serialization.
#[test]
fn test_const_tns_serde() -> anyhow::Result<()> {
    // Serialize - const namespace should be embedded in output XML
    let expected_xml = r#"<ex:Envelope xmlns:ex="https://my.example.org/"><ex:header>BASE_64_ENCODED_XML</ex:header></ex:Envelope>"#;
    let example = Example {
        header: Header {
            content: "BASE_64_ENCODED_XML".to_string(),
        },
        _xmlns: Default::default(),
    };
    let xml = raxb::ser::to_string(&example)?;
    assert_eq!(xml, expected_xml);

    // Deserialize - namespace URI is read from XML, const definition is not used
    let de: Example = raxb::de::from_str(expected_xml)?;
    assert_eq!(de.header.content, "BASE_64_ENCODED_XML");
    Ok(())
}

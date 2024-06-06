use raxb::{value::ConstStr, XmlDeserialize, XmlSerialize};

#[derive(Default, Debug, XmlSerialize, XmlDeserialize)]
#[raxb(root = b"a")]
pub struct A {
    #[raxb(name = b"product", ty = "attr", value = "H & D")]
    pub product: ConstStr,
    #[raxb(
        ns = b"xmlns",
        name = b"xsi",
        ty = "attr",
        value = "http://www.w3.org/2001/XMLSchema-instance"
    )]
    pub xmlns_xsi: ConstStr,
    #[raxb(
        ns = b"xsi",
        name = b"schemaLocation",
        ty = "attr",
        value = "https://myexample.org/examplev100 example.xsd"
    )]
    pub schema_location: ConstStr,
}

#[test]
fn test_const_str_serde() -> anyhow::Result<()> {
    let test_xml = r#"<a product="H &amp; D" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="https://myexample.org/examplev100 example.xsd"/>"#;
    let a1 = A::default();
    let s = raxb::ser::to_string(&a1)?;
    assert_eq!(s, test_xml);
    let a2: A = raxb::de::from_str(test_xml)?;
    assert_eq!(a2.product.as_ref(), "H & D");
    assert_eq!(
        a2.xmlns_xsi.as_ref(),
        "http://www.w3.org/2001/XMLSchema-instance"
    );
    assert_eq!(
        a2.schema_location.as_ref(),
        "https://myexample.org/examplev100 example.xsd"
    );
    Ok(())
}

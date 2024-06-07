use raxb::{XmlDeserialize, XmlSerialize};

#[derive(Default, Debug, XmlSerialize, XmlDeserialize, PartialEq, Eq)]
pub struct B {
    #[raxb(ty = "text")]
    text: String,
}

#[derive(Default, Debug, XmlSerialize, XmlDeserialize, PartialEq, Eq)]
pub struct C {
    #[raxb(ty = "text", default)]
    text: String,
}

#[derive(Default, Debug, XmlSerialize, XmlDeserialize, PartialEq, Eq)]
#[raxb(root = b"a")]
pub struct A {
    #[raxb(name = b"b", ty = "child", default)]
    pub b: B,
    #[raxb(name = b"c", ty = "child")]
    pub c: C,
    #[raxb(name = b"d", ty = "child", default)]
    pub d: String,
}

#[test]
fn test_const_str_serde() -> anyhow::Result<()> {
    let test_xml = r#"<a><b></b><c></c><d></d></a>"#;
    let a1 = A::default();
    let s = raxb::ser::to_string(&a1)?;
    assert_eq!(s, test_xml);
    let a2: A = raxb::de::from_str(test_xml)?;
    assert_eq!(a1, a2);
    Ok(())
}

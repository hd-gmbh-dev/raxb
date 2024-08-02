use raxb::{XmlDeserialize, XmlSerialize};

#[derive(Default, Debug, XmlDeserialize, XmlSerialize, PartialEq, Eq)]
#[raxb(root = b"a")]
pub struct A {
    #[raxb(name = b"d", ty = "child", default)]
    pub d: String,
    #[raxb(name = b"c", ty = "child", default)]
    pub c: String,
}

#[derive(Default, Debug, XmlDeserialize, XmlSerialize, PartialEq, Eq)]
#[raxb(root = b"b")]
pub struct B {
    #[raxb(name = b"d", ty = "child", default)]
    pub d: String,
}

#[derive(Debug, XmlDeserialize, XmlSerialize, PartialEq, Eq)]
enum E {
    #[xml(name = b"a")]
    A(A),
    #[xml(name = b"b")]
    B(B),
}

#[derive(Debug, XmlDeserialize, XmlSerialize, PartialEq, Eq)]
enum F {
    #[xml(name = b"a")]
    A(String),
    #[xml(name = b"b")]
    B,
}

#[cfg(feature = "trace")]
#[test_log::test]
fn test_enums_serde() -> anyhow::Result<()> {
    let test_xml1 = r#"<a><d>A</d><c>hello</c></a>"#;
    let v1: E = raxb::de::from_str(test_xml1)?;
    assert_eq!(v1, E::A(A { d: "A".to_string(), c: "hello".to_string() }));
    Ok(())
}

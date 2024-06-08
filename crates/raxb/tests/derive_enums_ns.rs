use quick_xml::{events::Event, name::ResolveResult};
use raxb::{de::XmlDeserializeError, ty::S, XmlDeserialize, XmlSerialize};

#[derive(Default, Debug, XmlDeserialize, XmlSerialize, PartialEq, Eq)]
#[xml(root = b"a")]
#[xml(tns(b"ns1", b"https://local.dev/example"))]
pub struct A {
    #[xml(ns = b"ns1", name = b"d", ty = "child", default)]
    pub d: String,
}

#[derive(Default, Debug, XmlDeserialize, XmlSerialize, PartialEq, Eq)]
#[xml(root = b"b")]
#[xml(tns(b"ns1", b"https://local.dev/example"))]
pub struct B {
    #[xml(ns = b"ns1", name = b"d", ty = "child", default)]
    pub d: String,
}

#[derive(Debug, XmlDeserialize, XmlSerialize, PartialEq, Eq)]
#[xml(tns(b"ns1", b"https://local.dev/example"))]
enum E {
    #[xml(ns = b"ns1", name = b"a")]
    A(A),
    #[xml(ns = b"ns1", name = b"b")]
    B(B),
}

#[test]
fn test_enums_with_ns_serde() -> anyhow::Result<()> {
    let test_xml1 = r#"<ns1:a xmlns:ns1="https://local.dev/example"><ns1:d>A</ns1:d></ns1:a>"#;
    let v1: E = raxb::de::from_str(test_xml1)?;
    let r1 = E::A(A { d: "A".to_string() });
    assert_eq!(v1, r1);
    assert_eq!(test_xml1, raxb::ser::to_string(&r1)?);
    let test_xml2 = r#"<ns1:b xmlns:ns1="https://local.dev/example"><ns1:d>B</ns1:d></ns1:b>"#;
    let v2: E = raxb::de::from_str(test_xml2)?;
    let r2 = E::B(B { d: "B".to_string() });
    assert_eq!(v2, r2);
    assert_eq!(test_xml2, raxb::ser::to_string(&r2)?);
    Ok(())
}

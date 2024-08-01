use quick_xml::{events::Event, name::ResolveResult};
use raxb::{de::XmlDeserializeError, ty::S, value::ConstStr, XmlDeserialize, XmlSerialize};

#[derive(Default, Debug, XmlDeserialize, XmlSerialize, PartialEq, Eq)]
#[xml(root = b"a")]
#[xml(tns(b"ns1", b"https://local.dev/example"))]
pub struct A {
    #[xml(
        ns = b"xmlns",
        name = b"ns1",
        ty = "attr",
        value = "https://local.dev/example"
    )]
    _xmlns: ConstStr,
    #[xml(ns = b"ns1", name = b"d", ty = "child", default)]
    pub d: String,
}

#[derive(Default, Debug, XmlDeserialize, XmlSerialize, PartialEq, Eq)]
#[xml(root = b"b")]
#[xml(tns(b"ns1", b"https://local.dev/example"))]
pub struct B {
    #[xml(
        ns = b"xmlns",
        name = b"ns1",
        ty = "attr",
        value = "https://local.dev/example"
    )]
    _xmlns: ConstStr,
    #[xml(ns = b"ns1", name = b"d", ty = "child", default)]
    pub d: String,
}

#[derive(Default, Debug, XmlDeserialize, XmlSerialize, PartialEq, Eq)]
#[xml(tns(b"ns1", b"https://local.dev/example"))]
enum E {
    #[xml(ns = b"ns1", name = b"a")]
    A(A),
    #[xml(ns = b"ns1", name = b"b")]
    B(B),
    #[default]
    #[xml(ns = b"ns1", name = b"none")]
    None,
}

#[derive(Default, Debug, XmlDeserialize, XmlSerialize, PartialEq, Eq)]
#[xml(tns(b"ns1", b"https://local.dev/example"))]
struct F {
    #[xml(ns = b"ns1", name = b"a", ty = "child", default)]
    pub a: String,
}

#[derive(Default, Debug, XmlDeserialize, XmlSerialize, PartialEq, Eq)]
#[xml(tns(b"ns1", b"https://local.dev/example"))]
struct G {
    #[xml(ns = b"ns1", name = b"d", ty = "child", default)]
    pub d: String,
}


#[derive(Default, Debug, XmlDeserialize, XmlSerialize, PartialEq, Eq)]
#[xml(tns(b"ns1", b"https://local.dev/example"))]
enum H {
    #[xml(ns = b"ns1", name = b"f")]
    F(F),
    #[xml(ns = b"ns1", name = b"g")]
    G(G),
    #[default]
    #[xml(ns = b"ns1", name = b"none")]
    None,
}

#[derive(Default, Debug, XmlDeserialize, XmlSerialize, PartialEq, Eq)]
#[xml(root = b"j")]
#[xml(tns(b"ns1", b"https://local.dev/example"))]
struct J {
    #[xml(
        ns = b"xmlns",
        name = b"ns1",
        ty = "attr",
        value = "https://local.dev/example"
    )]
    _xmlns: ConstStr,
    #[xml(ns = b"ns1", name = b"h", ty = "child")]
    h: H,
}

#[test]
fn test_enums_with_ns_serde() -> anyhow::Result<()> {
    let test_xml1 = r#"<ns1:a xmlns:ns1="https://local.dev/example"><ns1:d>A</ns1:d></ns1:a>"#;
    let v1: E = raxb::de::from_str(test_xml1)?;
    let r1 = E::A(A { d: "A".to_string(), ..Default::default() });
    assert_eq!(v1, r1);
    assert_eq!(test_xml1, raxb::ser::to_string(&r1)?);
    let test_xml2 = r#"<ns1:b xmlns:ns1="https://local.dev/example"><ns1:d>B</ns1:d></ns1:b>"#;
    let v2: E = raxb::de::from_str(test_xml2)?;
    let r2 = E::B(B { d: "B".to_string(), ..Default::default() });
    assert_eq!(v2, r2);
    assert_eq!(test_xml2, raxb::ser::to_string(&r2)?);
    Ok(())
}

#[test]
fn test_enum_child_with_ns_serde() -> anyhow::Result<()> {
    let test_xml1 = r#"<ns1:j xmlns:ns1="https://local.dev/example"><ns1:h><ns1:f><ns1:a>A</ns1:a></ns1:f></ns1:h></ns1:j>"#;
    let v1: J = raxb::de::from_str(test_xml1)?;
    let r1 = J { h: H::F(F { a: "A".to_string() }), ..Default::default() };
    assert_eq!(v1, r1);
    let xml = raxb::ser::to_string(&r1)?;
    let r2: J = raxb::de::from_str(&xml)?;
    assert_eq!(v1, r2);
    Ok(())
}

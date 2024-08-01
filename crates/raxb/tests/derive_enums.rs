use raxb::{de::XmlDeserializeError, ty::S, XmlDeserialize, XmlSerialize};

#[derive(Default, Debug, XmlDeserialize, XmlSerialize, PartialEq, Eq)]
#[raxb(root = b"a")]
pub struct A {
    #[raxb(name = b"d", ty = "child", default)]
    pub d: String,
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

#[test]
fn test_enums_serde() -> anyhow::Result<()> {
    let test_xml1 = r#"<a><d>A</d></a>"#;
    let v1: E = raxb::de::from_str(test_xml1)?;
    assert_eq!(v1, E::A(A { d: "A".to_string() }));
    let test_xml2 = r#"<b><d>B</d></b>"#;
    let v2: E = raxb::de::from_str(test_xml2)?;
    assert_eq!(v2, E::B(B { d: "B".to_string() }));
    Ok(())
}

#[test]
fn test_enums_serde_empty_tags() -> anyhow::Result<()> {
    let test_xml1 = r#"<a>A</a>"#;
    let v1: F = raxb::de::from_str(test_xml1)?;
    let r1 = F::A("A".to_string());
    assert_eq!(test_xml1, raxb::ser::to_string(&r1)?);
    assert_eq!(v1, r1);
    let test_xml2 = r#"<b/>"#;
    let v2: F = raxb::de::from_str(test_xml2)?;
    let r2 = F::B;
    assert_eq!(v2, r2);
    assert_eq!(test_xml2, raxb::ser::to_string(&r2)?);
    Ok(())
}

#[test]
fn test_missing_variant() -> anyhow::Result<()> {
    let test_xml1 = r#"<c><d>A</d></c>"#;
    let v1 = raxb::de::from_str::<E>(test_xml1);
    assert!(match v1 {
        Err(XmlDeserializeError::MissingVariant(S(v))) => {
            assert_eq!(v, b"'a'|'b'");
            true
        }
        _ => false,
    });
    Ok(())
}

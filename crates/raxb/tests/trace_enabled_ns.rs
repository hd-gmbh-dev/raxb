use raxb::{value::ConstStr, XmlDeserialize, XmlSerialize};

#[allow(dead_code)]
#[derive(Default, Debug, XmlDeserialize, XmlSerialize, PartialEq, Eq)]
#[xml(tns(b"ns1", b"https://local.dev/example"))]
struct F {
    #[xml(ns = b"ns1", name = b"a", ty = "child", default)]
    pub a: String,
}

#[allow(dead_code)]
#[derive(Default, Debug, XmlDeserialize, XmlSerialize, PartialEq, Eq)]
#[xml(tns(b"ns1", b"https://local.dev/example"))]
struct G {
    #[xml(ns = b"ns1", name = b"d", ty = "child", default)]
    pub d: String,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[cfg(feature = "trace")]
#[test_log::test]
fn test_enum_child_with_ns_serde() -> anyhow::Result<()> {
    let test_xml1 = r#"<ns1:j xmlns:ns1="https://local.dev/example">
        <ns1:h>
            <ns1:f>
                <ns1:a>A</ns1:a>
            </ns1:f>
        </ns1:h>
    </ns1:j>"#;
    let v1: J = raxb::de::from_str(test_xml1)?;
    let r1 = J {
        h: H::F(F { a: "A".to_string() }),
        ..Default::default()
    };
    assert_eq!(v1, r1);

    // println!("{}", raxb::ser::to_string(&H::F(F { a: "A".to_string() })).unwrap());

    Ok(())
}

#[cfg(feature = "trace")]
#[test_log::test]
fn test_enum_with_ns_serde() -> anyhow::Result<()> {
    let test_xml1 = r#"<ns1:f xmlns:ns1="https://local.dev/example"><ns1:a>A</ns1:a></ns1:f>"#;
    let v1: H = raxb::de::from_str(test_xml1)?;
    let r1 = H::F(F { a: "A".to_string() });
    assert_eq!(v1, r1);
    // println!("{}", raxb::ser::to_string(&H::F(F { a: "A".to_string() })).unwrap());

    Ok(())
}

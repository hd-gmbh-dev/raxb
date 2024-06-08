use quick_xml::{events::Event, name::ResolveResult};
use raxb::{
    de::{XmlDeserialize, XmlDeserializeError},
    ty::S,
    XmlDeserialize,
};

#[derive(Default, Debug, XmlDeserialize, PartialEq, Eq)]
#[raxb(root = b"a")]
pub struct A {
    #[raxb(name = b"d", ty = "child", default)]
    pub d: String,
}

#[derive(Default, Debug, XmlDeserialize, PartialEq, Eq)]
#[raxb(root = b"b")]
pub struct B {
    #[raxb(name = b"d", ty = "child", default)]
    pub d: String,
}

#[derive(Debug, PartialEq, Eq)]
enum E {
    A(A),
    B(B),
}

impl XmlDeserialize for E {
    fn is_enum() -> bool {
        true
    }

    fn root() -> Option<raxb::ty::XmlTag> {
        None
    }

    fn target_ns() -> Option<raxb::ty::XmlTargetNs> {
        None
    }

    fn xml_deserialize<R>(
        reader: &mut quick_xml::NsReader<R>,
        _target_ns: raxb::ty::XmlTag,
        _tag: raxb::ty::XmlTargetNs,
        _attributes: quick_xml::events::attributes::Attributes,
        _is_empty: bool,
    ) -> raxb::de::XmlDeserializeResult<Self>
    where
        Self: Sized,
        R: std::io::prelude::BufRead,
    {
        let mut result = Option::<E>::None;
        let mut buf = Vec::<u8>::new();
        loop {
            match reader.read_resolved_event_into(&mut buf)? {
                (ResolveResult::Unbound, Event::Start(e)) => match e.local_name().as_ref() {
                    root if Some(root) == A::root() => {
                        result = Some(E::A(A::xml_deserialize(
                            reader,
                            &[],
                            b"a",
                            e.attributes(),
                            false,
                        )?));
                        break;
                    }
                    root if Some(root) == B::root() => {
                        result = Some(E::B(B::xml_deserialize(
                            reader,
                            &[],
                            b"b",
                            e.attributes(),
                            false,
                        )?));
                        break;
                    }
                    _ => {
                        return Err(raxb::de::XmlDeserializeError::UnknownVariant(
                            String::from_utf8_lossy(e.name().as_ref()).to_string(),
                            S(b"'a' | 'b'"),
                        ));
                    }
                },
                (ResolveResult::Unbound, Event::Empty(e)) => match e.local_name().as_ref() {
                    root if Some(root) == A::root() => {
                        result = Some(E::A(A::xml_deserialize(
                            reader,
                            &[],
                            b"a",
                            e.attributes(),
                            true,
                        )?));
                        break;
                    }
                    root if Some(root) == B::root() => {
                        result = Some(E::B(B::xml_deserialize(
                            reader,
                            &[],
                            b"b",
                            e.attributes(),
                            true,
                        )?));
                        break;
                    }
                    _ => {
                        return Err(raxb::de::XmlDeserializeError::UnknownVariant(
                            String::from_utf8_lossy(e.name().as_ref()).to_string(),
                            S(b"'a' | 'b'"),
                        ));
                    }
                },
                (_, Event::Eof) => {
                    break;
                }
                _ => {}
            }
        }
        result.ok_or(XmlDeserializeError::MissingVariant(S(b"a | b")))
    }
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

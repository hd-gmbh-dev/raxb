use quick_xml::{events::Event, name::ResolveResult};
use raxb::de::{XmlDeserialize, XmlDeserializeError};
use raxb::ty::S;

#[derive(Debug)]
pub struct Envelope<T>
where
    T: XmlDeserialize + std::fmt::Debug,
{
    pub body: T,
}

impl<T> XmlDeserialize for Envelope<T>
where
    T: XmlDeserialize + std::fmt::Debug,
{
    fn root() -> Option<raxb::ty::XmlTag> {
        Some(b"Envelope")
    }

    fn target_ns() -> Option<raxb::ty::XmlTargetNs> {
        Some(b"http://schemas.xmlsoap.org/soap/envelope/")
    }

    fn xml_deserialize<R>(
        reader: &mut quick_xml::NsReader<R>,
        target_ns: raxb::ty::XmlTag,
        tag: raxb::ty::XmlTargetNs,
        _attributes: quick_xml::events::attributes::Attributes,
        _is_empty: bool,
    ) -> raxb::de::XmlDeserializeResult<Self>
    where
        Self: Sized,
        R: std::io::prelude::BufRead,
    {
        let target_ns = Self::target_ns().unwrap_or(target_ns);
        let mut buf = Vec::<u8>::new();
        let mut body = Option::<T>::None;
        loop {
            match reader.read_resolved_event_into(&mut buf)? {
                (ResolveResult::Bound(ns), Event::Start(ev)) if ns.as_ref() == target_ns => {
                    match ev.local_name().as_ref() {
                        b"Body" => {
                            body = Some(T::xml_deserialize(
                                reader,
                                target_ns,
                                b"Body",
                                ev.attributes(),
                                false,
                            )?);
                        }
                        _ => {
                            let mut buffer: Vec<u8> = Vec::<u8>::new();
                            reader.read_to_end_into(ev.name(), &mut buffer)?;
                        }
                    }
                }
                (ResolveResult::Bound(ns), Event::End(e))
                    if ns.as_ref() == target_ns && e.local_name().as_ref() == tag =>
                {
                    break;
                }
                (_, Event::Eof) => {
                    break;
                }
                _ => {}
            }
        }

        Ok(Self {
            body: body.ok_or(XmlDeserializeError::MissingElement(S(b"Body")))?,
        })
    }
}

#[derive(Debug)]
pub struct Header {
    pub content: String,
}

impl XmlDeserialize for Header {
    fn root() -> Option<raxb::ty::XmlTag> {
        Some(b"Example")
    }

    fn target_ns() -> Option<raxb::ty::XmlTargetNs> {
        Some(b"https://my.example.org/")
    }

    fn xml_deserialize<R>(
        reader: &mut quick_xml::NsReader<R>,
        target_ns: raxb::ty::XmlTag,
        tag: raxb::ty::XmlTargetNs,
        _attributes: quick_xml::events::attributes::Attributes,
        _is_empty: bool,
    ) -> raxb::de::XmlDeserializeResult<Self>
    where
        Self: Sized,
        R: std::io::prelude::BufRead,
    {
        let target_ns = Self::target_ns().unwrap_or(target_ns);
        let mut buf = Vec::<u8>::new();
        let mut content: Option<String> = Option::<String>::None;
        loop {
            match reader.read_resolved_event_into(&mut buf)? {
                (_, Event::Text(ev)) => content = Some(ev.unescape()?.to_string()),
                (ResolveResult::Bound(ns), Event::End(e))
                    if e.local_name().as_ref() == tag && ns.as_ref() == target_ns =>
                {
                    break;
                }
                (_, Event::Eof) => {
                    break;
                }
                _ => {}
            }
        }

        Ok(Self {
            content: content.ok_or(XmlDeserializeError::EmptyNode)?,
        })
    }
}

#[derive(Debug)]
pub struct Example {
    pub header: Header,
}

impl XmlDeserialize for Example {
    fn root() -> Option<raxb::ty::XmlTag> {
        Some(b"Example")
    }

    fn target_ns() -> Option<raxb::ty::XmlTargetNs> {
        Some(b"https://my.example.org/")
    }

    fn xml_deserialize<R>(
        reader: &mut quick_xml::NsReader<R>,
        target_ns: raxb::ty::XmlTargetNs,
        tag: raxb::ty::XmlTag,
        _attributes: quick_xml::events::attributes::Attributes,
        _is_empty: bool,
    ) -> raxb::de::XmlDeserializeResult<Self>
    where
        Self: Sized,
        R: std::io::prelude::BufRead,
    {
        let target_ns = Self::target_ns().unwrap_or(target_ns);
        let mut buf = Vec::<u8>::new();
        let mut header = Option::<Header>::None;
        loop {
            match reader.read_resolved_event_into(&mut buf)? {
                (ResolveResult::Bound(ns), Event::Start(ev)) if ns.as_ref() == target_ns => {
                    match ev.local_name().as_ref() {
                        b"header" => {
                            header = Some(Header::xml_deserialize(
                                reader,
                                target_ns,
                                b"header",
                                ev.attributes(),
                                false,
                            )?);
                        }
                        _ => {
                            let mut buffer: Vec<u8> = Vec::<u8>::new();
                            reader.read_to_end_into(ev.name(), &mut buffer)?;
                        }
                    }
                }
                (ResolveResult::Bound(ns), Event::End(e))
                    if e.local_name().as_ref() == tag && ns.as_ref() == target_ns =>
                {
                    break;
                }
                (_, Event::Eof) => {
                    break;
                }
                _ => {}
            }
        }

        Ok(Self {
            header: header.ok_or(XmlDeserializeError::MissingElement(S(b"Header")))?,
        })
    }
}

#[test]
fn test_deserialize_ns_manual() -> anyhow::Result<()> {
    let xml = r#"<SOAP-ENV:Envelope xmlns:SOAP-ENV="http://schemas.xmlsoap.org/soap/envelope/">
    <SOAP-ENV:Header/>
    <SOAP-ENV:Body xmlns:xsd="http://www.w3.org/2001/XMLSchema">
        <example:header xmlns:example="https://my.example.org/">BASE_64_ENCODED_XML</example:header>
    </SOAP-ENV:Body>
</SOAP-ENV:Envelope>"#;

    let envelope: Envelope<Example> = raxb::de::from_str(xml)?;
    eprintln!("{envelope:#?}");

    Ok(())
}

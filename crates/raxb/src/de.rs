use quick_xml::events::attributes::Attributes;
use quick_xml::events::Event;
use quick_xml::name::ResolveResult;
use quick_xml::NsReader;
use std::io::BufRead;
use std::num::ParseIntError;
use std::str::ParseBoolError;
use std::string::FromUtf8Error;
use thiserror::Error;

pub type XmlDeserializeResult<T> = Result<T, XmlDeserializeError>;
pub type XmlTag = &'static [u8];
pub type XmlTargetNs = &'static [u8];

#[derive(Clone)]
pub struct S(pub XmlTag);

impl From<XmlTag> for S {
    fn from(value: XmlTag) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for S {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(self.0))
    }
}

impl std::fmt::Debug for S {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", String::from_utf8_lossy(self.0))
    }
}

#[derive(Error, Debug)]
pub enum XmlDeserializeError {
    #[error(transparent)]
    Xml(#[from] quick_xml::Error),
    #[error(transparent)]
    Integer(#[from] ParseIntError),
    #[error(transparent)]
    Utf8String(#[from] FromUtf8Error),
    #[error(transparent)]
    Bool(#[from] ParseBoolError),
    #[error("empty element, try to add #[raxb(default)] attribute")]
    EmptyNode,
    #[error("missing root element name, try to implement 'fn root() -> XmlTag {{ b\"my-root-element-name\" }}'")]
    MissingRoot,
    #[error("missing element '{0}'")]
    MissingElement(S),
    #[error("missing attribute '{0}'")]
    MissingAttribute(S),
}

pub trait XmlDeserialize {
    fn root() -> Option<XmlTag> {
        None
    }

    fn target_ns() -> Option<XmlTargetNs> {
        None
    }

    fn xml_deserialize<R>(
        reader: &mut NsReader<R>,
        target_ns: XmlTag,
        tag: XmlTargetNs,
        attributes: Attributes,
        is_empty: bool,
    ) -> XmlDeserializeResult<Self>
    where
        Self: Sized,
        R: BufRead;
}

pub fn from_str<T>(s: &str) -> XmlDeserializeResult<T>
where
    T: XmlDeserialize,
{
    let mut rdr = quick_xml::NsReader::from_str(s);
    let mut buf = Vec::<u8>::new();
    let mut result = Option::<T>::None;
    let root = T::root().ok_or(XmlDeserializeError::MissingRoot)?;
    if let Some(target_ns) = T::target_ns() {
        loop {
            match rdr.read_resolved_event_into(&mut buf)? {
                (ResolveResult::Bound(tns), Event::Start(e)) => {
                    if e.local_name().as_ref() == root && tns.as_ref() == target_ns {
                        result = Some(T::xml_deserialize(
                            &mut rdr,
                            target_ns,
                            root,
                            e.attributes(),
                            false,
                        )?);
                    }
                }
                (ResolveResult::Bound(tns), Event::Empty(e)) => {
                    if e.local_name().as_ref() == root && tns.as_ref() == target_ns {
                        result = Some(T::xml_deserialize(
                            &mut rdr,
                            target_ns,
                            root,
                            e.attributes(),
                            true,
                        )?);
                    }
                }
                (_, Event::Eof) => {
                    break;
                }
                _ => {}
            }
        }
    } else {
        loop {
            match rdr.read_resolved_event_into(&mut buf)? {
                (ResolveResult::Unbound, Event::Start(e)) => {
                    if e.local_name().as_ref() == root {
                        result = Some(T::xml_deserialize(
                            &mut rdr,
                            &[],
                            root,
                            e.attributes(),
                            false,
                        )?);
                    }
                }
                (ResolveResult::Unbound, Event::Empty(e)) => {
                    if e.local_name().as_ref() == root {
                        result = Some(T::xml_deserialize(
                            &mut rdr,
                            &[],
                            root,
                            e.attributes(),
                            true,
                        )?);
                    }
                }
                (_, Event::Eof) => {
                    break;
                }
                _ => {}
            }
        }
    }
    result.ok_or(XmlDeserializeError::MissingElement(root.into()))
}

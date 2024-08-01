use quick_xml::events::attributes::Attributes;
use quick_xml::events::Event;
use quick_xml::name::ResolveResult;
use quick_xml::NsReader;
use std::num::ParseIntError;
use std::str::ParseBoolError;
use std::string::FromUtf8Error;
use std::{io::BufRead, num::ParseFloatError};
use thiserror::Error;

use crate::ty::{XmlTag, XmlTargetNs, S};

pub type XmlDeserializeResult<T> = Result<T, XmlDeserializeError>;

#[derive(Error, Debug)]
pub enum XmlDeserializeError {
    #[error(transparent)]
    Xml(#[from] quick_xml::Error),
    #[error(transparent)]
    XmlEscapeError(#[from] quick_xml::escape::EscapeError),
    #[error(transparent)]
    Integer(#[from] ParseIntError),
    #[error(transparent)]
    Float(#[from] ParseFloatError),
    #[error(transparent)]
    Utf8String(#[from] FromUtf8Error),
    #[error(transparent)]
    Bool(#[from] ParseBoolError),
    #[error("empty element, try to add #[raxb(default)] attribute")]
    EmptyNode,
    #[error("missing root element name, try to implement 'fn root() -> XmlTag {{ b\"my-root-element-name\" }}'")]
    MissingRoot,
    #[error("missing element name, expected one of {0}")]
    MissingVariant(S),
    #[error("unknown variant '{0}', expected one of {1}")]
    UnknownVariant(String, S),
    #[error("missing element '{0}'")]
    MissingElement(S),
    #[error("missing attribute '{0}'")]
    MissingAttribute(S),
}

pub trait XmlDeserialize {
    fn is_enum() -> bool {
        false
    }

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

pub fn deserialize_with_reader<T, R>(mut rdr: NsReader<R>) -> XmlDeserializeResult<T>
where
    T: XmlDeserialize,
    R: BufRead,
{
    rdr.trim_text(true);
    rdr.check_comments(false);
    rdr.expand_empty_elements(false);
    if T::is_enum() {
        if let Some(target_ns) = T::target_ns() {
            return T::xml_deserialize(&mut rdr, target_ns, &[], Attributes::new("", 0), false);
        } else {
            return T::xml_deserialize(&mut rdr, &[], &[], Attributes::new("", 0), false);
        }
    }
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
                    } else {
                        let mut buf = Vec::<u8>::new();
                        rdr.read_to_end_into(e.name(), &mut buf)?;
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
                    } else {
                        let mut buf = Vec::<u8>::new();
                        rdr.read_to_end_into(e.name(), &mut buf)?;
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

pub fn from_str<T>(s: &str) -> XmlDeserializeResult<T>
where
    T: XmlDeserialize,
{
    deserialize_with_reader(quick_xml::NsReader::from_str(s))
}

pub fn from_reader<R, T>(s: R) -> XmlDeserializeResult<T>
where
    R: BufRead,
    T: XmlDeserialize,
{
    deserialize_with_reader(quick_xml::NsReader::<R>::from_reader(s))
}

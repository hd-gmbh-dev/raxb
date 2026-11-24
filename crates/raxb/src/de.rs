use quick_xml::events::attributes::Attributes;
use quick_xml::events::{BytesRef, Event};
use quick_xml::name::ResolveResult;
use quick_xml::{NsReader, XmlVersion};
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
    #[error(transparent)]
    Encoding(#[from] quick_xml::encoding::EncodingError),
    #[error("unrecognized entity '&{0};'")]
    UnknownEntity(String),
}

/// Resolves a `&name;` or `&#nn;` reference to its character.
pub fn resolve_entity_ref(r: &BytesRef<'_>) -> XmlDeserializeResult<String> {
    if let Some(c) = r.resolve_char_ref()? {
        return Ok(c.to_string());
    }
    let name = r.decode()?;
    quick_xml::escape::resolve_predefined_entity(&name)
        .map(str::to_string)
        .ok_or_else(|| XmlDeserializeError::UnknownEntity(name.to_string()))
}

/// Reads the text content of the element `tag` until its end tag, resolving
/// character and entity references.
pub fn read_text_content<R: BufRead>(
    reader: &mut NsReader<R>,
    tag: &[u8],
) -> XmlDeserializeResult<String> {
    let mut value = String::new();
    let mut buf = Vec::<u8>::new();
    loop {
        match reader.read_resolved_event_into(&mut buf)? {
            (_, Event::Text(t)) => value.push_str(&t.xml_content(XmlVersion::Explicit1_0)?),
            (_, Event::GeneralRef(r)) => value.push_str(&resolve_entity_ref(&r)?),
            (_, Event::Start(e)) => {
                let mut skipped = Vec::<u8>::new();
                reader.read_to_end_into(e.name(), &mut skipped)?;
            }
            (_, Event::End(e)) if e.local_name().as_ref() == tag => break,
            (_, Event::Eof) => break,
            _ => {}
        }
    }
    Ok(value.trim().to_string())
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
    rdr.config_mut().trim_text(false);
    rdr.config_mut().check_comments = false;
    rdr.config_mut().expand_empty_elements = false;
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

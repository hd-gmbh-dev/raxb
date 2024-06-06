use std::{io::Cursor, str::Utf8Error, string::FromUtf8Error};

use quick_xml::Writer;
use thiserror::Error;

use crate::ty::XmlTag;

#[derive(Error, Debug)]

pub enum XmlSerializeError {
    #[error(transparent)]
    Xml(#[from] quick_xml::Error),
    #[error(transparent)]
    Utf8String(#[from] FromUtf8Error),
    #[error(transparent)]
    Utf8Error(#[from] Utf8Error),
    #[error("missing root element name, try to implement 'fn root() -> XmlTag {{ b\"my-root-element-name\" }}'")]
    MissingRoot,
}

pub type XmlSerializeResult<T> = Result<T, XmlSerializeError>;

pub trait XmlSerialize {
    fn root() -> Option<XmlTag> {
        None
    }

    fn xml_serialize<W: std::io::Write>(
        &self,
        tag: &str,
        writer: &mut Writer<W>,
    ) -> XmlSerializeResult<()>;
}

pub fn to_string<T>(value: &T) -> XmlSerializeResult<String>
where
    T: XmlSerialize,
{
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let name = std::str::from_utf8(T::root().ok_or(XmlSerializeError::MissingRoot)?)?;
    value.xml_serialize(name, &mut writer)?;
    Ok(String::from_utf8(writer.into_inner().into_inner())?)
}

pub fn to_string_pretty<T>(value: &T) -> XmlSerializeResult<String>
where
    T: XmlSerialize,
{
    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);
    let name = std::str::from_utf8(T::root().ok_or(XmlSerializeError::MissingRoot)?)?;
    value.xml_serialize(name, &mut writer)?;
    Ok(String::from_utf8(writer.into_inner().into_inner())?)
}

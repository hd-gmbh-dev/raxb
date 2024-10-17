use quick_xml::events::BytesStart;
use quick_xml::events::{attributes::AttrError, Event};
use quick_xml::Reader;
use std::borrow::Cow;
use std::num::ParseFloatError;
use std::num::ParseIntError;
use std::str::ParseBoolError;
use std::string::FromUtf8Error;
use thiserror::Error;

use crate::ty::{XmlTag, S};

pub type XmlBorrowResult<T> = Result<T, XmlBorrowError>;

#[derive(Error, Debug)]
pub enum XmlBorrowError {
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
    #[error(transparent)]
    Attr(#[from] AttrError),
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

pub type XmlValue<'a> = Cow<'a, str>;

pub struct Pointer<'a> {
    path: Vec<Option<BytesStart<'a>>>,
    idx: usize,
}

impl<'a> Pointer<'a> {
    pub fn new() -> Self {
        Self {
            path: vec![],
            idx: 0,
        }
    }

    pub fn visit(&mut self, ptr: BytesStart<'a>) {
        if self.path.len() < self.idx + 1 {
            self.path.push(Some(ptr));
        } else {
            self.path[self.idx] = Some(ptr);
        }
        self.idx += 1;
    }

    pub fn leave(&mut self) {
        if self.idx > 0 {
            self.idx -= 1;
            self.path[self.idx] = None;
        }
    }
}

impl<'a> PartialEq<&[&str]> for Pointer<'a> {
    fn eq(&self, other: &&[&str]) -> bool {
        let path = &self.path[0..self.idx];
        if path.len() != other.len() {
            return false;
        }
        for (p, s) in path.iter().zip(other.iter()) {
            if p.is_none() {
                return false;
            }
            if p.as_ref().unwrap().local_name().as_ref() != s.as_bytes() {
                return false;
            }
        }
        true
    }
}

struct PointerPath<'a, 'b>(&'a [Option<BytesStart<'b>>]);

impl<'a, 'b> std::fmt::Debug for PointerPath<'a, 'b> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dl = f.debug_list();
        for v in self.0.iter() {
            if let Some(v) = v {
                dl.entry(&String::from_utf8_lossy(v.local_name().as_ref()));
            }
        }
        dl.finish()
    }
}

impl<'a> std::fmt::Debug for Pointer<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("Pointer");
        s.field("idx", &self.idx);
        s.field("path", &PointerPath(&self.path));
        s.finish()
    }
}

pub trait XmlBorrow<'a> {
    fn root() -> Option<XmlTag> {
        None
    }
    fn xml_borrow(
        reader: &mut Reader<&'a [u8]>,
        bytes_start: Option<BytesStart<'a>>,
    ) -> XmlBorrowResult<Self>
    where
        Self: Sized;
}

pub fn borrow_with_reader<'a, T>(mut rdr: Reader<&'a [u8]>) -> XmlBorrowResult<T>
where
    T: XmlBorrow<'a>,
{
    let mut result = Option::<T>::None;
    rdr.config_mut().check_comments = false;
    rdr.config_mut().expand_empty_elements = false;
    rdr.config_mut().check_end_names = false;
    let root = T::root();
    loop {
        let ev = rdr.read_event()?;
        match ev {
            Event::Start(bytes_start) => {
                if let Some(root) = root {
                    if bytes_start.local_name().as_ref() == root {
                        result = Some(T::xml_borrow(&mut rdr, Some(bytes_start))?)
                    } else {
                        break;
                    }
                } else {
                    result = Some(T::xml_borrow(&mut rdr, Some(bytes_start))?)
                }
            }
            Event::Eof => {
                break;
            }
            _ => {}
        }
    }
    result.ok_or(if let Some(root) = root {
        XmlBorrowError::MissingElement(root.into())
    } else {
        XmlBorrowError::MissingRoot
    })
}

pub fn from_str<'a, T>(s: &'a str) -> XmlBorrowResult<T>
where
    T: XmlBorrow<'a>,
{
    borrow_with_reader(quick_xml::Reader::from_str(s))
}

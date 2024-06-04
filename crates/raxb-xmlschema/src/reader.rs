use byteorder::ReadBytesExt;
use lz4_flex::{block::DecompressError, decompress_size_prepended};
use std::{collections::HashMap, ops::Range, string::FromUtf8Error};
use thiserror::Error;

use crate::cnst::{Order, MAGIC_BYTE};
pub type SchemaBundleIndex = HashMap<String, Range<usize>>;

#[derive(Error, Debug)]
pub enum ReaderError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Utf8Error(#[from] FromUtf8Error),
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
    #[cfg(feature = "writer")]
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    DecompressError(#[from] DecompressError),
    #[error("no entrypoint")]
    NoEntrypoint,
    #[error("invalid file format")]
    InvalidFormat,
    #[error("invalid file header")]
    InvalidHead,
}

pub type ReaderResult<T> = Result<T, ReaderError>;

fn read_string(rdr: &mut std::io::Cursor<Vec<u8>>) -> ReaderResult<String> {
    let s_length = rdr.read_u32::<Order>()? as usize;
    let mut buf = Vec::with_capacity(s_length);
    for _ in 0..s_length {
        buf.push(rdr.read_u8()?);
    }
    Ok(String::from_utf8(buf)?)
}

pub trait XmlSchemaResolver {
    fn entrypoint(&self) -> &[u8];
    fn resolve(&self, s: &str) -> Option<&[u8]>;
}

#[derive(Default, Debug)]
pub struct SchemaBundleHeader {
    entrypoint: Range<usize>,
    name: String,
    target_ns: String,
    index: SchemaBundleIndex,
}

#[derive(Default)]
pub struct SchemaBundle {
    pub header: SchemaBundleHeader,
    buffer: Vec<u8>,
}

impl SchemaBundle {
    pub fn from_slice(b: &[u8]) -> ReaderResult<Self> {
        let mut index = SchemaBundleIndex::default();
        let mut rdr = std::io::Cursor::new(decompress_size_prepended(b)?);
        let magic_byte = rdr.read_u32::<Order>()?;
        if magic_byte != MAGIC_BYTE {
            return Err(ReaderError::InvalidFormat);
        }
        let head_size = rdr.read_u64::<Order>()? as usize;
        let name = read_string(&mut rdr)?;
        let target_ns = read_string(&mut rdr)?;
        let mut pos = 4 + 8 + 4 + name.len() + 4 + target_ns.len();
        let mut entrypoint = 0..0usize;
        loop {
            match pos {
                p if p == head_size => {
                    break;
                }
                p if p > head_size => {
                    return Err(ReaderError::InvalidHead);
                }
                _ => {
                    let is_entrypoint = rdr.read_u8()? == 1;
                    let start = rdr.read_u64::<Order>()? as usize;
                    let end = rdr.read_u64::<Order>()? as usize;
                    if is_entrypoint {
                        entrypoint = start..end;
                    }
                    let name = read_string(&mut rdr)?;
                    pos += 1 + 8 + 8 + 4 + name.len();
                    index.insert(name, start..end);
                }
            }
        }
        let buffer = rdr.into_inner().split_off(head_size);
        Ok(SchemaBundle {
            header: SchemaBundleHeader {
                entrypoint,
                name,
                target_ns,
                index,
            },
            buffer,
        })
    }

    pub fn name(&self) -> &str {
        &self.header.name
    }

    pub fn target_ns(&self) -> &str {
        &self.header.target_ns
    }
}

impl XmlSchemaResolver for SchemaBundle {
    fn entrypoint(&self) -> &[u8] {
        &self.buffer[self.header.entrypoint.start..self.header.entrypoint.end]
    }
    fn resolve(&self, name: &str) -> Option<&[u8]> {
        let result = self
            .header
            .index
            .get(name)
            .map(|e| &self.buffer[e.start..e.end]);
        if result.is_none() {
            eprintln!(
                "'{name}' not found, available schemas are -> {:#?}",
                self.header.index
            );
        }
        result
    }
}

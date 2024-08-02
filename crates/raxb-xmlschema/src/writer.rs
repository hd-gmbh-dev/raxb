use crate::cnst::{Order, MAGIC_BYTE};
use lz4_flex::block::compress_prepend_size;
use std::{collections::BTreeMap, path::PathBuf, string::FromUtf8Error};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WriterError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Utf8Error(#[from] FromUtf8Error),
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
    #[cfg(feature = "writer")]
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("no entrypoint")]
    NoEntrypoint,
    #[error("invalid file format")]
    InvalidFormat,
    #[error("invalid file header")]
    InvalidHead,
}

pub type WriterResult<T> = Result<T, WriterError>;

fn create_uuid(b: &[u8]) -> String {
    uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_URL, b)
        .as_simple()
        .to_string()
}

pub fn create_filepath<P: AsRef<std::path::Path>>(path: P, target_namespace: &str) -> PathBuf {
    path.as_ref()
        .join(format!("{}.xsdb", create_uuid(target_namespace.as_bytes())))
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SchemaLocation {
    Path(PathBuf),
    Url(url::Url),
}

impl SchemaLocation {
    pub fn get_content(&self, cache_dir: &std::path::Path) -> WriterResult<String> {
        Ok(match self {
            Self::Url(url) => {
                let cache_name = format!("{url}");
                let cached_file = cache_dir.join(create_uuid(cache_name.as_bytes()));
                if cached_file.exists() {
                    return Ok(std::fs::read_to_string(&cached_file)?);
                }
                let result = reqwest::blocking::get(url.as_ref())?.text()?;
                std::fs::write(&cached_file, &result)?;
                result
            }
            Self::Path(path) => std::fs::read_to_string(path)?,
        })
    }

    pub fn try_join(&self, other: &str) -> WriterResult<Self> {
        Ok(match self {
            Self::Url(u) => Self::Url(u.join(other)?),
            Self::Path(u) => Self::Path(u.parent().unwrap().join(other)),
        })
    }
}

impl std::fmt::Display for SchemaLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Url(u) => u.fmt(f),
            Self::Path(u) => u.display().fmt(f),
        }
    }
}

impl std::str::FromStr for SchemaLocation {
    type Err = WriterError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s.starts_with("http") {
            SchemaLocation::Url(s.parse()?)
        } else {
            SchemaLocation::Path(PathBuf::from(s))
        })
    }
}

#[derive(Default)]
pub struct SchemaWriter {
    w: std::io::Cursor<Vec<u8>>,
}

#[derive(Debug)]
pub struct SchemaEntry {
    target_namespace: String,
    entrypoint: bool,
    content: String,
}

impl SchemaEntry {
    pub fn new(target_namespace: String, entrypoint: bool, content: String) -> Self {
        Self {
            target_namespace,
            entrypoint,
            content,
        }
    }
}

impl SchemaWriter {
    pub fn write(mut self, map: BTreeMap<SchemaLocation, SchemaEntry>) -> WriterResult<Vec<u8>> {
        eprintln!("write {map:#?}");
        use byteorder::WriteBytesExt;
        use std::io::Write;
        let m: Vec<(String, SchemaEntry)> = map
            .into_iter()
            .map(|(k, v)| {
                let s = match k {
                    SchemaLocation::Path(p) => p.file_name().unwrap().to_str().unwrap().to_string(),
                    SchemaLocation::Url(u) => u.to_string(),
                };
                (s, v)
            })
            .collect::<Vec<_>>();
        let (entrypoint_name, v) = m
            .iter()
            .find(|v| v.1.entrypoint)
            .ok_or(WriterError::NoEntrypoint)?;
        let initial_headsize = 4 + 8 + 4 + entrypoint_name.len() + 4 + v.target_namespace.len();
        let head_size = m.iter().fold(initial_headsize, |state, (e, _)| {
            state + 1 + 8 + 8 + 4 + e.len()
        });
        self.w.write_u32::<Order>(MAGIC_BYTE)?;
        self.w.write_u64::<Order>(head_size as u64)?;
        self.w.write_u32::<Order>(entrypoint_name.len() as u32)?;
        self.w.write_all(entrypoint_name.as_bytes())?;
        self.w.write_u32::<Order>(v.target_namespace.len() as u32)?;
        self.w.write_all(v.target_namespace.as_bytes())?;
        let mut pos = 0;
        for (name, v) in m.iter() {
            let end = pos + v.content.len();
            self.w.write_u8(if v.entrypoint { 1 } else { 0 })?; // is entrypoint?
            self.w.write_u64::<Order>(pos as u64)?; // start
            self.w.write_u64::<Order>(end as u64)?; // end
            self.w.write_u32::<Order>(name.len() as u32)?; // name length
            self.w.write_all(name.as_bytes())?;
            pos = end;
        }
        for (_, v) in m.iter() {
            self.w.write_all(v.content.as_bytes())?;
        }
        self.w.flush()?;
        Ok(compress_prepend_size(&self.w.into_inner()))
    }
}

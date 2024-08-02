use raxb::{de::XmlDeserializeError, XmlDeserialize};
use raxb_xmlschema::writer::{
    create_filepath, SchemaEntry, SchemaLocation, SchemaWriter, WriterError,
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BuildError {
    #[error(transparent)]
    Fmt(#[from] std::fmt::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    WriterError(#[from] WriterError),
    #[error(transparent)]
    XmlDeserialize(#[from] XmlDeserializeError),
}

pub type BuildResult<T> = Result<T, BuildError>;

#[derive(Debug, XmlDeserialize)]
#[raxb(tns(b"xs", b"http://www.w3.org/2001/XMLSchema"))]
pub struct XsdImportOrInclude {
    #[raxb(name = b"schemaLocation", ty = "attr")]
    pub schema_location: String,
}

#[derive(Debug, XmlDeserialize)]
#[raxb(root = b"schema")]
#[raxb(tns(b"xs", b"http://www.w3.org/2001/XMLSchema"))]
pub struct Xsd {
    #[raxb(name = b"targetNamespace", ty = "attr")]
    pub tns: String,
    #[raxb(ns = b"xs", name = b"include", ty = "sfc")]
    pub includes: Vec<XsdImportOrInclude>,
    #[raxb(ns = b"xs", name = b"import", ty = "sfc")]
    pub imports: Vec<XsdImportOrInclude>,
}

pub struct Schema {
    entrypoint: bool,
    location: SchemaLocation,
    content: String,
    filename: Option<std::path::PathBuf>,
}

pub struct XmlSchemaRegistry {
    out: PathBuf,
    cache_dir: PathBuf,
    input: Vec<Schema>,
}

impl XmlSchemaRegistry {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            out: path.as_ref().to_owned(),
            cache_dir: path.as_ref().join("cache"),
            input: Default::default(),
        }
    }

    pub fn with_cache_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        path.as_ref().clone_into(&mut self.cache_dir);
        self
    }

    pub fn register(&mut self, url_or_path: &str) -> BuildResult<()> {
        self.input.push(Schema {
            location: url_or_path.parse()?,
            content: String::default(),
            entrypoint: true,
            filename: None,
        });
        Ok(())
    }

    pub fn register_with_filename<P: AsRef<Path>>(
        &mut self,
        url_or_path: &str,
        filename: P,
    ) -> BuildResult<()> {
        self.input.push(Schema {
            location: url_or_path.parse()?,
            content: String::default(),
            entrypoint: true,
            filename: Some(filename.as_ref().to_owned()),
        });
        Ok(())
    }

    fn try_save(self, log: &mut String) -> BuildResult<()> {
        let mut counter = 0;
        let cache_dir = &self.cache_dir;
        let out_dir = self.out.join("out");
        std::fs::create_dir_all(cache_dir)?;
        for input in self.input {
            let mut root = input.location.clone();
            let mut schemas: BTreeMap<SchemaLocation, SchemaEntry> = BTreeMap::default();
            let mut schema_list = vec![input];
            let mut filepath = Option::None;
            while let Some(mut schema) = schema_list.pop() {
                if schemas.contains_key(&schema.location) {
                    continue;
                }
                {
                    use std::fmt::Write;
                    writeln!(log, "{counter}: {}", schema.location)?;
                }
                schema.content = schema.location.get_content(cache_dir)?;
                eprintln!("{}", schema.content);
                let Xsd {
                    tns,
                    imports,
                    includes,
                } = raxb::de::from_str(&schema.content)?;
                eprintln!("{imports:#?}");
                eprintln!("{includes:#?}");
                if schema.entrypoint {
                    if let Some(filename) = schema.filename.as_ref() {
                        filepath = Some(filename.to_owned());
                    } else {
                        filepath = Some(create_filepath(&out_dir, &tns));
                    }
                }
                for XsdImportOrInclude { schema_location } in
                    imports.into_iter().chain(includes.into_iter())
                {
                    schema_list.push(Schema {
                        location: {
                            if !schema_location.starts_with("http") {
                                root.try_join(&schema_location)
                                    .unwrap_or(schema_location.parse()?)
                            } else {
                                schema_location.parse()?
                            }
                        },
                        content: Default::default(),
                        entrypoint: false,
                        filename: None,
                    });
                }
                root = schema.location.clone();
                schemas.insert(
                    schema.location,
                    SchemaEntry::new(tns, schema.entrypoint, schema.content),
                );
                counter += 1;
            }

            if let Some(filepath) = filepath {
                std::fs::write(filepath, SchemaWriter::default().write(schemas)?)?;
            }
        }
        Ok(())
    }

    pub fn save(self) -> BuildResult<()> {
        std::fs::create_dir_all(&self.out)?;
        std::fs::create_dir_all(self.out.join("out"))?;
        let mut log = String::default();
        let out = self.out.join("xml-validate-build.log");
        let result = self.try_save(&mut log);
        std::fs::write(out, log)?;
        result
    }
}

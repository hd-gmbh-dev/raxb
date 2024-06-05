#![allow(dead_code)]

use raxb::{XmlDeserialize, XmlSerialize};

#[derive(Debug, XmlDeserialize, XmlSerialize)]
#[raxb(root = b"Envelope")]
#[raxb(tns(b"SOAP", b"https://schemas.xmlsoap.org/soap/envelope/"))]
pub struct Envelope<T>
where
    T: raxb::de::XmlDeserialize + raxb::ser::XmlSerialize + std::fmt::Debug,
{
    #[raxb(ns = b"SOAP", name = b"Header", ty = "sfc")]
    pub header: bool,
    #[raxb(ns = b"SOAP", name = b"Body", ty = "child")]
    pub body: T,
}

#[derive(Debug, XmlDeserialize, XmlSerialize)]
pub struct Header {
    #[raxb(ty = "text")]
    pub content: String,
}

#[derive(Debug, XmlDeserialize, XmlSerialize)]
#[raxb(tns(b"ex", b"https://my.example.org/"))]
pub struct Example {
    #[raxb(ns = b"ex", name = b"header", ty = "child")]
    pub header: Header,
}

#[test]
fn test_serialize_ns_derive() -> anyhow::Result<()> {
    let xml = raxb::ser::to_string(&Envelope::<Example> {
        header: true,
        body: Example {
            header: Header {
                content: "BASE_64_ENCODED_XML".to_string(),
            },
        },
    })?;
    assert_eq!(
        xml,
        r#"<SOAP:Envelope xmlns:SOAP="https://schemas.xmlsoap.org/soap/envelope/"><SOAP:Header/><SOAP:Body xmlns:ex="https://my.example.org/"><ex:header>BASE_64_ENCODED_XML</ex:header></SOAP:Body></SOAP:Envelope>"#
    );
    Ok(())
}

#[test]
fn test_deserialize_ns_with_derive_macro() -> anyhow::Result<()> {
    let xml = r#"<SOAP-ENV:Envelope xmlns:SOAP-ENV="https://schemas.xmlsoap.org/soap/envelope/">
    <SOAP-ENV:Header/>
    <SOAP-ENV:Body xmlns:xsd="http://www.w3.org/2001/XMLSchema">
        <example:header xmlns:example="https://my.example.org/">BASE_64_ENCODED_XML</example:header>
    </SOAP-ENV:Body>
</SOAP-ENV:Envelope>"#;
    let envelope: Envelope<Example> = raxb::de::from_str(xml)?;
    eprintln!("{envelope:#?}");

    Ok(())
}

#[derive(Debug, XmlDeserialize)]
pub struct XsdImportOrInclude {
    #[raxb(name = b"schemaLocation", ty = "attr")]
    pub schema_location: String,
}

#[derive(Debug, XmlDeserialize)]
#[raxb(root = b"schema")]
#[raxb(tns(b"xs", b"http://www.w3.org/2001/XMLSchema"))]
pub struct Xsd {
    #[raxb(ns = b"xs", name = b"include", ty = "sfc")]
    pub includes: Vec<XsdImportOrInclude>,
    #[raxb(ns = b"xs", name = b"import", ty = "sfc")]
    pub imports: Vec<XsdImportOrInclude>,
}

#[test]
fn test_deserialize_ns_with_derive_macro_with_decl() -> anyhow::Result<()> {
    let xml = r#"<?xml version="1.0" encoding="utf-8" ?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema" targetNamespace="https://local.dev/example" xmlns:example="https://local.dev/example" elementFormDefault="qualified">
    <xs:import schemaLocation="./example2.xsd" namespace="https://local.dev/example2" />
    <xs:include  schemaLocation="./example1.xsd" />
</xs:schema>"#;
    let xsd: Xsd = raxb::de::from_str(xml)?;
    eprintln!("{xsd:#?}");
    Ok(())
}

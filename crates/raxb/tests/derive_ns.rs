#![allow(dead_code)]

use raxb::{value::ConstStr, XmlDeserialize, XmlSerialize};

#[derive(Debug, XmlDeserialize, XmlSerialize)]
#[xml(root = b"Envelope")]
#[xml(tns(b"SOAP", b"https://schemas.xmlsoap.org/soap/envelope/"))]
pub struct Envelope<T>
where
    T: raxb::de::XmlDeserialize + raxb::ser::XmlSerialize + std::fmt::Debug,
{
    #[xml(
        default,
        ns = b"xmlns",
        name = b"SOAP",
        ty = "attr",
        value = "https://schemas.xmlsoap.org/soap/envelope/"
    )]
    _xmlns: ConstStr,
    #[xml(ns = b"SOAP", name = b"Header", ty = "sfc")]
    pub header: bool,
    #[xml(ns = b"SOAP", name = b"Body", ty = "child")]
    pub body: T,
}

#[derive(Debug, XmlDeserialize, XmlSerialize)]
pub struct Header {
    #[xml(ty = "text")]
    pub content: String,
}

#[derive(Debug, XmlDeserialize, XmlSerialize)]
#[xml(tns(b"ex", b"https://my.example.org/"))]
pub struct Example {
    #[xml(
        default,
        ns = b"xmlns",
        name = b"ex",
        ty = "attr",
        value = "https://my.example.org/"
    )]
    _xmlns: ConstStr,
    #[xml(ns = b"ex", name = b"header", ty = "child")]
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
            _xmlns: Default::default(),
        },
        _xmlns: Default::default(),
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
    #[xml(name = b"schemaLocation", ty = "attr")]
    pub schema_location: String,
}

#[derive(Debug, XmlDeserialize)]
#[xml(root = b"schema")]
#[xml(tns(b"xs", b"http://www.w3.org/2001/XMLSchema"))]
pub struct Xsd {
    #[xml(ns = b"xs", name = b"include", ty = "sfc")]
    pub includes: Vec<XsdImportOrInclude>,
    #[xml(ns = b"xs", name = b"import", ty = "sfc")]
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

#[test]
fn test_deserialize_real_schema() -> anyhow::Result<()> {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           targetNamespace="https://gitlab.opencode.de/akdb/xoev/xwasser/-/raw/main/V0_5_0"
           version="0.5.0"
           elementFormDefault="qualified"
           attributeFormDefault="unqualified">
   <xs:include schemaLocation="xwasser-administration.xsd"/>
   <xs:include schemaLocation="xwasser-basisdatentypen.xsd"/>
   <xs:include schemaLocation="xwasser-basisnachricht.xsd"/>
   <xs:include schemaLocation="xwasser-baukasten-erweiterung.xsd"/>
   <xs:include schemaLocation="xwasser-baukasten.xsd"/>
   <xs:include schemaLocation="xwasser-rueckweisung.xsd"/>
   <xs:include schemaLocation="xwasser-vorgang.xsd"/>
   <xs:include schemaLocation="xwasser-weiterleitung.xsd"/>
   <xs:include schemaLocation="xwasser-weiterleitungsnachrichten.xsd"/>
</xs:schema>"#;
    let xsd: Xsd = raxb::de::from_str(xml)?;
    eprintln!("{xsd:#?}");
    Ok(())
}

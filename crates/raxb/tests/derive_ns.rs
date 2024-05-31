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

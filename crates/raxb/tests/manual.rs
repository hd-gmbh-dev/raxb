#![allow(clippy::single_match)]

use quick_xml::{
    escape::escape,
    events::{attributes::Attributes, BytesText, Event},
    name::ResolveResult,
    NsReader,
};

use raxb::{
    de::{XmlDeserialize, XmlDeserializeError, XmlDeserializeResult},
    ser::{XmlSerialize, XmlSerializeError},
    ty::{XmlTag, XmlTargetNs, S},
};
use std::io::BufRead;

#[derive(Debug)]
pub struct F {
    pub h: Option<String>,
    pub j: String,
}

impl XmlDeserialize for F {
    fn xml_deserialize<R>(
        reader: &mut NsReader<R>,
        _target_ns: XmlTag,
        tag: XmlTargetNs,
        _attributes: Attributes,
        _is_empty: bool,
    ) -> XmlDeserializeResult<Self>
    where
        Self: Sized,
        R: BufRead,
    {
        let mut buf = Vec::<u8>::new();
        let mut h = Option::<String>::None;
        let mut j = Option::<String>::None;

        loop {
            match reader.read_resolved_event_into(&mut buf)? {
                (ResolveResult::Unbound, Event::Start(ev)) => match ev.local_name().as_ref() {
                    b"h" => {
                        let mut buffer: Vec<u8> = Vec::<u8>::new();
                        if let (ResolveResult::Unbound, Event::Text(t)) =
                            reader.read_resolved_event_into(&mut buffer)?
                        {
                            h = Some(t.unescape()?.to_string());
                        }
                    }
                    b"j" => {
                        let mut buffer: Vec<u8> = Vec::<u8>::new();
                        if let (ResolveResult::Unbound, Event::Text(t)) =
                            reader.read_resolved_event_into(&mut buffer)?
                        {
                            j = Some(t.unescape()?.to_string());
                        }
                    }
                    _ => {
                        let mut buffer: Vec<u8> = Vec::<u8>::new();
                        reader.read_to_end_into(ev.name(), &mut buffer)?;
                    }
                },
                (ResolveResult::Unbound, Event::End(e)) if e.local_name().as_ref() == tag => {
                    break;
                }
                (ResolveResult::Unbound, Event::Eof) => {
                    break;
                }
                _ => {}
            }
        }

        Ok(Self {
            h,
            j: j.ok_or(XmlDeserializeError::MissingElement(S(b"j")))?,
        })
    }
}

impl XmlSerialize for F {
    fn xml_serialize<W: std::io::Write>(
        &self,
        tag: &str,
        writer: &mut quick_xml::Writer<W>,
    ) -> raxb::ser::XmlSerializeResult<()> {
        writer
            .create_element(tag)
            .write_inner_content::<_, XmlSerializeError>(|writer| {
                if let Some(v) = self.h.as_ref() {
                    writer
                        .create_element("h")
                        .write_text_content(BytesText::from_escaped(escape(v)))?;
                }
                writer
                    .create_element("j")
                    .write_text_content(BytesText::from_escaped(escape(&self.j)))?;
                Ok(())
            })?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct D {
    pub name: String,
    pub e: Vec<i32>,
    pub f: Vec<F>,
}

impl XmlDeserialize for D {
    fn xml_deserialize<R>(
        reader: &mut NsReader<R>,
        target_ns: XmlTag,
        tag: XmlTargetNs,
        attributes: Attributes,
        _is_empty: bool,
    ) -> XmlDeserializeResult<Self>
    where
        Self: Sized,
        R: BufRead,
    {
        let mut buf = Vec::<u8>::new();

        let mut name = Option::<String>::None;
        for attr in attributes.flatten() {
            match attr.key.local_name().as_ref() {
                b"name" => {
                    name = Some(String::from_utf8(attr.value.to_vec())?);
                }
                _ => {}
            }
        }

        let mut e = Vec::<i32>::default();
        let mut f = Vec::<F>::default();

        loop {
            match reader.read_resolved_event_into(&mut buf)? {
                (ResolveResult::Unbound, Event::Start(ev)) => match ev.local_name().as_ref() {
                    b"e" => {
                        let mut buffer: Vec<u8> = Vec::<u8>::new();
                        if let (ResolveResult::Unbound, Event::Text(t)) =
                            reader.read_resolved_event_into(&mut buffer)?
                        {
                            let v = t.unescape()?.parse::<i32>()?;
                            e.push(v);
                        }
                    }
                    b"f" => {
                        f.push(F::xml_deserialize(
                            reader,
                            target_ns,
                            b"f",
                            ev.attributes(),
                            false,
                        )?);
                    }
                    _ => {
                        let mut buffer: Vec<u8> = Vec::<u8>::new();
                        reader.read_to_end_into(ev.name(), &mut buffer)?;
                    }
                },
                (ResolveResult::Unbound, Event::End(e)) if e.local_name().as_ref() == tag => {
                    break;
                }
                (ResolveResult::Unbound, Event::Eof) => {
                    break;
                }
                _ => {}
            }
        }

        Ok(Self {
            name: name.ok_or(XmlDeserializeError::MissingAttribute(S(b"name")))?,
            e,
            f,
        })
    }
}

impl XmlSerialize for D {
    fn xml_serialize<W: std::io::Write>(
        &self,
        tag: &str,
        writer: &mut quick_xml::Writer<W>,
    ) -> raxb::ser::XmlSerializeResult<()> {
        writer
            .create_element(tag)
            .with_attribute(("name", self.name.as_str()))
            .write_inner_content::<_, XmlSerializeError>(|writer| {
                for v in self.e.iter() {
                    writer
                        .create_element("e")
                        .write_text_content(BytesText::from_escaped(escape(&v.to_string())))?;
                }
                for v in self.f.iter() {
                    v.xml_serialize("f", writer)?;
                }
                Ok(())
            })?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct A {
    pub id: String,
    pub b: bool,
    pub c: String,
    pub d: D,
}

impl XmlDeserialize for A {
    fn root() -> Option<XmlTag> {
        Some(b"a")
    }

    fn xml_deserialize<R>(
        reader: &mut NsReader<R>,
        target_ns: XmlTag,
        tag: XmlTargetNs,
        attributes: Attributes,
        _is_empty: bool,
    ) -> XmlDeserializeResult<Self>
    where
        Self: Sized,
        R: BufRead,
    {
        let mut buf = Vec::<u8>::new();

        let mut id = Option::<String>::None;
        for attr in attributes.flatten() {
            match attr.key.local_name().as_ref() {
                b"id" => {
                    id = Some(String::from_utf8(attr.value.to_vec())?);
                }
                _ => {}
            }
        }

        let mut b = Option::<bool>::None;
        let mut c = Option::<String>::None;
        let mut d = Option::<D>::None;

        loop {
            match reader.read_resolved_event_into(&mut buf)? {
                (ResolveResult::Unbound, Event::Start(e)) => match e.local_name().as_ref() {
                    b"c" => {
                        let mut buffer: Vec<u8> = Vec::<u8>::new();
                        if let (ResolveResult::Unbound, Event::Text(t)) =
                            reader.read_resolved_event_into(&mut buffer)?
                        {
                            c = Some(t.unescape()?.to_string());
                        }
                    }
                    b"d" => {
                        d = Some(D::xml_deserialize(
                            reader,
                            target_ns,
                            b"d",
                            e.attributes(),
                            false,
                        )?)
                    }
                    _ => {
                        let mut buffer: Vec<u8> = Vec::<u8>::new();
                        reader.read_to_end_into(e.name(), &mut buffer)?;
                    }
                },
                (ResolveResult::Unbound, Event::Empty(e)) => match e.local_name().as_ref() {
                    b"b" => {
                        b = Some(true);
                    }
                    _ => {}
                },
                (ResolveResult::Unbound, Event::End(e)) if e.local_name().as_ref() == tag => {
                    break;
                }
                (_, Event::Eof) => {
                    break;
                }
                _ => {}
            }
        }
        Ok(Self {
            id: id.ok_or(XmlDeserializeError::MissingAttribute(S(b"id")))?,
            b: b.ok_or(XmlDeserializeError::MissingElement(S(b"b")))?,
            c: c.ok_or(XmlDeserializeError::MissingElement(S(b"c")))?,
            d: d.ok_or(XmlDeserializeError::MissingElement(S(b"d")))?,
        })
    }
}

impl XmlSerialize for A {
    fn root() -> Option<XmlTag> {
        Some(b"a")
    }

    fn xml_serialize<W: std::io::Write>(
        &self,
        tag: &str,
        writer: &mut quick_xml::Writer<W>,
    ) -> raxb::ser::XmlSerializeResult<()> {
        writer
            .create_element(tag)
            .with_attribute(("id", self.id.as_str()))
            .write_inner_content::<_, XmlSerializeError>(|writer| {
                if self.b {
                    writer.create_element("b").write_empty()?;
                }
                writer
                    .create_element("c")
                    .write_text_content(BytesText::new(&self.c))?;
                self.d.xml_serialize("d", writer)?;
                Ok(())
            })?;
        Ok(())
    }
}

#[test]
fn test_serialize_manual() -> anyhow::Result<()> {
    let a = A {
        id: "root".to_string(),
        b: true,
        c: "foo".to_string(),
        d: D {
            name: "foobar".to_string(),
            e: vec![1, 2, 3],
            f: vec![
                F {
                    h: Some("bar1".to_string()),
                    j: "baz2".to_string(),
                },
                F {
                    h: None,
                    j: "baz".to_string(),
                },
            ],
        },
    };

    let xml = raxb::ser::to_string(&a)?;
    assert_eq!(
        r#"<a id="root"><b/><c>foo</c><d name="foobar"><e>1</e><e>2</e><e>3</e><f><h>bar1</h><j>baz2</j></f><f><j>baz</j></f></d></a>"#,
        xml
    );
    Ok(())
}

#[test]
fn test_deserialize_manual() -> anyhow::Result<()> {
    let xml = r#"<a id="root">
        <b/>
        <c>foo</c>
        <d name="foobar">
            <e>1</e>
            <e>2</e>
            <e>3</e>
            <f>
                <h>bar1</h>
                <j>baz2</j>
            </f>
            <f>
                <j>baz</j>
            </f>
        </d>
    </a>"#;
    let a = raxb::de::from_str::<A>(xml)?;
    assert_eq!(a.id, "root");
    assert!(a.b);
    assert_eq!(a.c, "foo");
    assert_eq!(a.d.name, "foobar");
    assert_eq!(a.d.e, vec![1, 2, 3]);
    assert_eq!(a.d.f.first().unwrap().h.as_ref().unwrap(), "bar1");
    assert_eq!(a.d.f.first().unwrap().j, "baz2");
    assert!(a.d.f.get(1).unwrap().h.is_none());
    assert_eq!(a.d.f.get(1).unwrap().j, "baz");
    Ok(())
}

#[test]
fn test_deserialize_manual_skipping_unknown_fields() -> anyhow::Result<()> {
    let xml = r#"<a id="root">
        <b/>
        <c>foo</c>
        <y>
            unknown
        </y>
        <d name="foobar">
            <e>1</e>
            <e>2</e>

            <z>
                <x>1</x>
                <x>2</x>
                <x>3</x>
            </z>
            <e>3</e>
            <f>
                <h>bar1</h>
                <j>baz2</j>
                <k>
                    <f>
                        <j>recurse</j>
                    </f>
                </k>
            </f>
            <f>
                <j>baz</j>
            </f>
        </d>
    </a>"#;
    let a = raxb::de::from_str::<A>(xml)?;
    assert!(a.b);
    assert_eq!(a.c, "foo");
    assert_eq!(a.d.name, "foobar");
    assert_eq!(a.d.e, vec![1, 2, 3]);
    assert_eq!(a.d.f.first().unwrap().h.as_ref().unwrap(), "bar1");
    assert_eq!(a.d.f.first().unwrap().j, "baz2");
    assert!(a.d.f.get(1).unwrap().h.is_none());
    assert_eq!(a.d.f.get(1).unwrap().j, "baz");
    Ok(())
}

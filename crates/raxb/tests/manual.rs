#![allow(clippy::single_match)]

use quick_xml::{
    events::{attributes::Attributes, Event},
    name::ResolveResult,
    NsReader,
};
use raxb::de::{XmlDeserialize, XmlDeserializeError, XmlDeserializeResult, XmlTag, XmlTargetNs, S};
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

#[derive(Debug)]
pub struct A {
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
        _attributes: Attributes,
        _is_empty: bool,
    ) -> XmlDeserializeResult<Self>
    where
        Self: Sized,
        R: BufRead,
    {
        let mut buf = Vec::<u8>::new();

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
            b: b.ok_or(XmlDeserializeError::MissingElement(S(b"b")))?,
            c: c.ok_or(XmlDeserializeError::MissingElement(S(b"c")))?,
            d: d.ok_or(XmlDeserializeError::MissingElement(S(b"d")))?,
        })
    }
}

#[test]
fn test_simple_xml() -> anyhow::Result<()> {
    let xml = r#"<a>
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
fn test_simple_skip_unknown_xml() -> anyhow::Result<()> {
    let xml = r#"<a>
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

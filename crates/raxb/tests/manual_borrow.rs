#![allow(clippy::single_match)]

use quick_xml::{
    events::{BytesStart, Event},
    Reader,
};

use raxb::{
    ty::S,
    zde::{Pointer, XmlBorrow, XmlBorrowError, XmlBorrowResult, XmlValue},
};

#[derive(Debug)]
pub struct F<'a> {
    pub h: Option<XmlValue<'a>>,
    pub j: Option<XmlValue<'a>>,
}

impl<'a> XmlBorrow<'a> for F<'a> {
    fn xml_borrow(
        reader: &mut Reader<&'a [u8]>,
        bytes_start: Option<BytesStart<'a>>,
    ) -> XmlBorrowResult<Self> {
        let mut h: Option<XmlValue> = None;
        let mut j: Option<XmlValue> = None;

        let h_path = &["h"][..];
        let j_path = &["j"][..];

        let mut p = Pointer::new();
        loop {
            let ev = reader.read_event()?;
            match ev {
                Event::Start(bytes_start) => {
                    p.visit(bytes_start);
                }
                Event::Text(bytes_text) => {
                    if p == h_path {
                        h = Some(bytes_text.unescape()?);
                    }
                    if p == j_path {
                        j = Some(bytes_text.unescape()?);
                    }
                }
                Event::End(bytes_end) => {
                    if let Some(bytes_start) = bytes_start.as_ref() {
                        let end = bytes_start.to_end();
                        if bytes_end == end {
                            break;
                        }
                    }
                    p.leave();
                }
                Event::Eof => break,
                _ => {}
            }
        }
        Ok(Self { h, j })
    }
}

#[derive(Debug)]
pub struct D<'a> {
    pub name: XmlValue<'a>,
    pub e: Vec<i32>,
    pub f: Vec<F<'a>>,
}

impl<'a> XmlBorrow<'a> for D<'a> {
    fn xml_borrow(
        reader: &mut Reader<&'a [u8]>,
        bytes_start: Option<BytesStart<'a>>,
    ) -> XmlBorrowResult<Self> {
        let mut name: Option<XmlValue> = None;
        let mut e: Vec<i32> = Default::default();
        let mut f: Vec<F> = Default::default();

        let e_path = &["e"][..];

        if let Some(bytes_start) = bytes_start.as_ref() {
            let attrs = bytes_start.attributes();
            for attr in attrs {
                let attr = attr?;
                match attr.key.local_name().as_ref() {
                    b"name" => {
                        let attr = attr
                            .to_owned()
                            .decode_and_unescape_value(reader.decoder())?;
                        name = Some(std::borrow::Cow::Owned(attr.to_string()));
                    }
                    _ => {}
                }
            }
        }

        let mut p = Pointer::new();
        loop {
            let ev = reader.read_event()?;
            match ev {
                Event::Start(bytes_start) => match bytes_start.local_name().as_ref() {
                    b"f" => {
                        f.push(F::xml_borrow(reader, Some(bytes_start))?);
                    }
                    _ => {
                        p.visit(bytes_start);
                    }
                },
                Event::Text(bytes_text) => {
                    if p == e_path {
                        e.push(bytes_text.unescape()?.parse::<i32>()?);
                    }
                }
                Event::End(bytes_end) => {
                    if let Some(bytes_start) = bytes_start.as_ref() {
                        let end = bytes_start.to_end();
                        if bytes_end == end {
                            break;
                        }
                    }
                    p.leave();
                }
                Event::Eof => break,
                _ => {}
            }
        }
        Ok(Self {
            name: name.ok_or(XmlBorrowError::MissingAttribute(S(b"name")))?,
            e,
            f,
        })
    }
}

#[derive(Debug)]
pub struct A<'a> {
    pub id: XmlValue<'a>,
    pub b: bool,
    pub b2: bool,
    pub c: XmlValue<'a>,
    pub d: Option<D<'a>>,
}

impl<'a> XmlBorrow<'a> for A<'a> {
    fn xml_borrow(
        reader: &mut Reader<&'a [u8]>,
        bytes_start: Option<BytesStart<'a>>,
    ) -> XmlBorrowResult<Self> {
        let mut id: Option<XmlValue> = None;
        let mut c: Option<XmlValue> = None;
        let mut d = Option::<D>::None;
        let mut b = false;
        let mut b2 = false;

        let c_path = &["c"][..];
        let b2_path = &["b2"][..];
        let mut p = Pointer::new();
        if let Some(bytes_start) = bytes_start.as_ref() {
            let attrs = bytes_start.attributes();
            for attr in attrs {
                let attr = attr?;
                match attr.key.local_name().as_ref() {
                    b"id" => {
                        let attr = attr
                            .to_owned()
                            .decode_and_unescape_value(reader.decoder())?;
                        id = Some(std::borrow::Cow::Owned(attr.to_string()));
                    }
                    _ => {}
                }
            }
        }
        loop {
            let ev = reader.read_event()?;
            match ev {
                Event::Start(bytes_start) => match bytes_start.local_name().as_ref() {
                    b"d" => {
                        d = Some(D::xml_borrow(reader, Some(bytes_start))?);
                    }
                    _ => {
                        p.visit(bytes_start);
                    }
                },
                Event::Empty(bytes_start) => {
                    if bytes_start.local_name().as_ref() == b"b" {
                        b = true;
                    }
                }
                Event::Text(bytes_text) => {
                    if p == c_path {
                        c = Some(bytes_text.unescape()?);
                    }
                    if p == b2_path {
                        b2 = &bytes_text.unescape()?.to_ascii_lowercase() == "true";
                    }
                }
                Event::End(bytes_end) => {
                    if let Some(bytes_start) = bytes_start.as_ref() {
                        let end = bytes_start.to_end();
                        if bytes_end == end {
                            break;
                        }
                    }
                    p.leave();
                }
                Event::Eof => break,
                _ => {}
            }
        }
        Ok(Self {
            id: id.ok_or(XmlBorrowError::MissingAttribute(S(b"id")))?,
            c: c.ok_or(XmlBorrowError::MissingElement(S(b"c")))?,
            b,
            b2,
            d,
        })
    }
}

#[test]
fn test_borrow_manual() -> anyhow::Result<()> {
    let xml = r#"<a id="root">
        <b/>
        <b2>true</b2>
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
    let a = raxb::zde::from_str::<A>(xml)?;
    assert_eq!(a.id, "root");
    assert!(a.b);
    assert!(a.b2);
    assert_eq!(a.c, "foo");
    assert!(a.d.is_some());
    assert_eq!(a.d.as_ref().unwrap().name, "foobar");
    assert_eq!(a.d.as_ref().unwrap().e, vec![1, 2, 3]);
    assert_eq!(
        a.d.as_ref().unwrap().f.first().unwrap().h.as_ref().unwrap(),
        "bar1"
    );
    assert_eq!(
        a.d.as_ref().unwrap().f.first().unwrap().j.as_ref().unwrap(),
        "baz2"
    );
    assert!(a.d.as_ref().unwrap().f.get(1).unwrap().h.is_none());
    assert_eq!(
        a.d.as_ref().unwrap().f.get(1).unwrap().j.as_ref().unwrap(),
        "baz"
    );
    Ok(())
}

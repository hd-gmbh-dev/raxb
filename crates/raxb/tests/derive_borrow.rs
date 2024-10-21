#![allow(clippy::single_match)]
use raxb::{zde::XmlValue, XmlBorrow};

#[derive(Debug, XmlBorrow)]
pub struct F<'a> {
    pub h: Option<XmlValue<'a>>,
    pub j: Option<XmlValue<'a>>,
}

#[derive(Debug, XmlBorrow)]
pub struct D<'a> {
    #[xml(ty = "attr")]
    pub name: XmlValue<'a>,
    pub e: Vec<i32>,
    #[xml(ty = "child")]
    pub f: Vec<F<'a>>,
}

#[derive(Debug, XmlBorrow)]
pub struct A<'a> {
    #[xml(ty = "attr")]
    pub id: XmlValue<'a>,
    #[xml(default, ty = "sfc")]
    pub b: bool,
    pub b2: bool,
    #[xml(path = ["c", "x", "y"])]
    pub c: XmlValue<'a>,
    #[xml(ty = "child")]
    pub d: Option<D<'a>>,
}

#[test]
fn test_borrow_manual() -> anyhow::Result<()> {
    let xml = r#"<a id="root">
        <b/>
        <b2>true</b2>
        <c><x><y>foo</y></x></c>
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
    eprintln!("{a:#?}");
    assert_eq!(a.id, "root");
    assert!(a.b);
    assert!(a.b2);
    assert_eq!(a.c, "foo");
    assert!(a.d.is_some());
    assert_eq!(a.d.as_ref().unwrap().name, "foobar");
    assert_eq!(a.d.as_ref().unwrap().e, vec![1, 2, 3]);
    assert_eq!(a.d.as_ref().unwrap().f.first().unwrap().h.as_ref().unwrap(), "bar1");
    assert_eq!(a.d.as_ref().unwrap().f.first().unwrap().j.as_ref().unwrap(), "baz2");
    assert!(a.d.as_ref().unwrap().f.get(1).unwrap().h.is_none());
    assert_eq!(a.d.as_ref().unwrap().f.get(1).unwrap().j.as_ref().unwrap(), "baz");
    Ok(())
}

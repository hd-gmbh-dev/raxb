use raxb::{XmlDeserialize, XmlSerialize};

#[derive(Debug, XmlDeserialize, XmlSerialize)]
#[raxb(root = b"k")]
pub struct K {
    #[raxb(name = b"id", ty = "attr")]
    pub id: Option<String>,
    #[raxb(name = b"n", ty = "attr")]
    pub n: i32,
    #[raxb(ty = "text")]
    pub content: String,
}

#[derive(Debug, XmlDeserialize, XmlSerialize)]
#[raxb(root = b"f")]
pub struct F {
    #[raxb(name = b"h", ty = "child")]
    pub h: Option<String>,
    #[raxb(name = b"j", ty = "child")]
    pub j: String,
}

#[derive(Debug, XmlDeserialize, XmlSerialize)]
#[raxb(root = b"d")]
pub struct D {
    #[raxb(name = b"name", ty = "attr")]
    pub name: String,
    #[raxb(name = b"e", ty = "child")]
    pub e: Vec<i32>,
    #[raxb(name = b"f", ty = "child")]
    pub f: Vec<F>,
    #[raxb(name = b"k", ty = "child")]
    pub k: Vec<K>,
    #[raxb(name = b"d", ty = "child")]
    pub d: Vec<D>,
}

#[derive(Debug, XmlDeserialize, XmlSerialize)]
#[raxb(root = b"a")]
pub struct A {
    #[raxb(name = b"id", ty = "attr")]
    pub id: String,
    #[raxb(name = b"b", ty = "sfc")]
    pub b: bool,
    #[raxb(name = b"c", ty = "child")]
    pub c: String,
    #[raxb(name = b"d", ty = "child")]
    pub d: D,
}

#[test]
fn test_serialize_derive() -> anyhow::Result<()> {
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
            k: vec![K {
                content: "k content 1".to_string(),
                n: 32,
                id: Some("one".to_string()),
            }],
            d: vec![],
        },
    };
    let xml = raxb::ser::to_string(&a)?;
    assert_eq!(
        r#"<a id="root"><b/><c>foo</c><d name="foobar"><e>1</e><e>2</e><e>3</e><f><h>bar1</h><j>baz2</j></f><f><j>baz</j></f><k id="one" n="32">k content 1</k></d></a>"#,
        xml
    );
    Ok(())
}

#[test]
fn test_deserialize_with_derive_macro() -> anyhow::Result<()> {
    let xml = r#"<a id="root">
        <b/>
        <c>foo</c>
        <d name="foobar">
            <e>1</e>
            <e>2</e>
            <e>3</e>
            <d name="child">
                <e>4</e>
                <e>5</e>
                <e>6</e>
            </d>
            <k id="one" n="32">k content 1</k>
            <k id="two" n="64">k content 2</k>
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
    assert_eq!(a.d.d.first().unwrap().name, "child");
    assert_eq!(a.d.d.first().unwrap().e, vec![4, 5, 6]);
    assert_eq!(a.d.k.first().unwrap().id.as_ref().unwrap(), "one");
    assert_eq!(a.d.k.first().unwrap().n, 32);
    assert_eq!(a.d.k.get(1).unwrap().id.as_ref().unwrap(), "two");
    assert_eq!(a.d.k.get(1).unwrap().n, 64);
    Ok(())
}

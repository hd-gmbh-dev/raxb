use crate::symbol::*;

use strum::EnumString;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::Meta::{self, NameValue, Path};
use syn::Variant;

#[allow(non_camel_case_types)]
#[derive(Debug, Default, PartialEq, EnumString)]
pub enum BuiltInType {
    #[default]
    Unknown,
    String,
    bool,
    f32,
    f64,
    u8,
    u16,
    u32,
    u64,
    i8,
    i16,
    i32,
    i64,
}

#[derive(Debug, Default, PartialEq, EnumString)]
pub enum BuiltInConstType {
    #[default]
    Unknown,
    ConstStr,
}

impl BuiltInType {
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String)
    }
    pub fn is_bool(&self) -> bool {
        matches!(self, Self::bool)
    }
    pub fn is_number(&self) -> bool {
        matches!(
            self,
            Self::f32
                | Self::f64
                | Self::u8
                | Self::u16
                | Self::u32
                | Self::u64
                | Self::i8
                | Self::i16
                | Self::i32
                | Self::i64
        )
    }
}

#[allow(dead_code)]
pub struct Container<'a> {
    pub struct_fields: Vec<StructField<'a>>, // Struct fields
    pub enum_variants: Vec<EnumVariant<'a>>,
    pub original: &'a syn::DeriveInput,
    pub root: Option<syn::LitByteStr>,
    pub tns: Option<(syn::LitByteStr, syn::LitByteStr)>,
}

impl<'a> Container<'a> {
    pub fn is_enum(&self) -> bool {
        !self.enum_variants.is_empty()
    }

    pub fn validate(&self) {
        if self.root.is_some() && self.is_enum() {
            panic!("for clarity, enum should not have the root attribute. please use a struct to wrap the enum and set its type to untag")
        }
    }

    pub fn from_ast(item: &'a syn::DeriveInput, _derive: Derive) -> Container<'a> {
        let mut root = Option::<syn::LitByteStr>::None;
        let mut tns = Option::<(syn::LitByteStr, syn::LitByteStr)>::None;
        for meta_item in item
            .attrs
            .iter()
            .flat_map(get_xmlserde_meta_items)
            .flatten()
        {
            match meta_item {
                NameValue(m) if m.path == ROOT => {
                    let s = get_lit_byte_str(&m.value).expect("parse root failed");
                    root = Some(s.clone());
                }
                Meta::List(l) if l.path == TNS => {
                    let strs = l
                        .parse_args_with(Punctuated::<syn::LitByteStr, Comma>::parse_terminated)
                        .unwrap();
                    let mut iter = strs.iter();
                    let first = iter.next().expect("tns should have 2 arguments");
                    let second = iter.next().expect("tns should have 2 arguments");
                    if iter.next().is_some() {
                        panic!("tns should have 2 arguments")
                    }
                    tns = Some((first.clone(), second.clone()));
                }
                _ => panic!("unexpected"),
            }
        }
        match &item.data {
            syn::Data::Struct(ds) => {
                let fields = ds
                    .fields
                    .iter()
                    .filter_map(StructField::from_ast)
                    .collect::<Vec<_>>();
                Container {
                    struct_fields: fields,
                    enum_variants: vec![],
                    original: item,
                    root,
                    tns,
                }
            }
            syn::Data::Enum(e) => {
                let variants = e
                    .variants
                    .iter()
                    .map(EnumVariant::from_ast)
                    .collect::<Vec<_>>();
                Container {
                    struct_fields: vec![],
                    enum_variants: variants,
                    original: item,
                    root,
                    tns,
                }
            }
            syn::Data::Union(_) => panic!("Only support struct and enum type, union is found"),
        }
    }
}

pub struct FieldsSummary<'a> {
    pub children: Vec<StructField<'a>>,
    pub text: Option<StructField<'a>>,
    pub attrs: Vec<StructField<'a>>,
    pub self_closed_children: Vec<StructField<'a>>,
    pub untags: Vec<StructField<'a>>,
    pub any: Option<StructField<'a>>,
    pub xmlns: Option<StructField<'a>>,
}

impl<'a> FieldsSummary<'a> {
    pub fn from_fields(fields: Vec<StructField<'a>>) -> Self {
        let mut result = FieldsSummary {
            children: vec![],
            text: None,
            attrs: vec![],
            self_closed_children: vec![],
            untags: vec![],
            any: None,
            xmlns: None,
        };
        fields.into_iter().for_each(|f| match f.ty {
            EleType::Attr => result.attrs.push(f),
            EleType::Child => result.children.push(f),
            EleType::Text => result.text = Some(f),
            EleType::SelfClosedChild => result.self_closed_children.push(f),
            EleType::Untag => result.untags.push(f),
            EleType::Any => result.any = Some(f),
            EleType::XmlNs => result.xmlns = Some(f),
        });
        result
    }
}

pub struct StructField<'a> {
    pub ty: EleType,
    pub name: Option<syn::LitByteStr>,
    pub original: &'a syn::Field,
    pub generic: Generic<'a>,
    pub ns: Option<syn::LitByteStr>,
    pub value: Option<syn::LitStr>,
    pub default: bool,
}

impl<'a> StructField<'a> {
    pub fn from_ast(f: &'a syn::Field) -> Option<Self> {
        let mut name = Option::<syn::LitByteStr>::None;
        let mut ns = Option::<syn::LitByteStr>::None;
        let mut value = Option::<syn::LitStr>::None;
        let mut ty = Option::<EleType>::None;
        let mut default = false;
        let generic = get_generics(&f.ty);
        for meta_item in f.attrs.iter().flat_map(get_xmlserde_meta_items).flatten() {
            match meta_item {
                NameValue(m) if m.path == NAME => {
                    if let Ok(s) = get_lit_byte_str(&m.value) {
                        name = Some(s.clone());
                    }
                }
                NameValue(m) if m.path == NS => {
                    if let Ok(s) = get_lit_byte_str(&m.value) {
                        ns = Some(s.clone());
                    }
                }
                NameValue(m) if m.path == VALUE => {
                    if let Ok(s) = get_lit_str(&m.value) {
                        value = Some(s.clone());
                    }
                }
                NameValue(m) if m.path == TYPE => {
                    if let Ok(s) = get_lit_str(&m.value) {
                        let t = match s.value().as_str() {
                            "attr" => EleType::Attr,
                            "child" => EleType::Child,
                            "text" => EleType::Text,
                            "sfc" => EleType::SelfClosedChild,
                            "untag" => EleType::Untag,
                            "any" => EleType::Any,
                            "xmlns" => EleType::XmlNs,
                            _ => panic!("invalid type"),
                        };
                        ty = Some(t);
                    }
                }
                Path(p) if p == DEFAULT => {
                    default = true;
                }
                _ => panic!("unexpected"),
            }
        }
        if let Some(ty) = ty {
            Some(StructField {
                ty,
                name,
                original: f,
                generic,
                ns,
                value,
                default,
            })
        } else if f.ident.is_none() {
            Some(StructField {
                ty: EleType::Text,
                name,
                original: f,
                generic,
                ns,
                value,
                default,
            })
        } else {
            None
        }
    }

    pub fn is_required(&self) -> bool {
        matches!(self.generic, Generic::None)
    }
}

#[allow(dead_code)]
pub struct EnumVariant<'a> {
    pub name: Option<syn::LitByteStr>,
    pub ident: &'a syn::Ident,
    pub ty: Option<&'a syn::Type>,
    pub ele_type: EleType,
}

impl<'a> EnumVariant<'a> {
    pub fn from_ast(v: &'a Variant) -> Self {
        let mut name = Option::<syn::LitByteStr>::None;
        let mut ele_type = EleType::Child;
        for meta_item in v.attrs.iter().flat_map(get_xmlserde_meta_items).flatten() {
            match meta_item {
                NameValue(m) if m.path == NAME => {
                    if let Ok(s) = get_lit_byte_str(&m.value) {
                        name = Some(s.clone());
                    }
                }
                NameValue(m) if m.path == TYPE => {
                    if let Ok(s) = get_lit_str(&m.value) {
                        let t = match s.value().as_str() {
                            "child" => EleType::Child,
                            "text" => EleType::Text,
                            _ => panic!("invalid type in enum, should be `text` or `child` only"),
                        };
                        ele_type = t;
                    }
                }
                _ => panic!("unexpected attribute"),
            }
        }
        if v.fields.len() > 1 {
            panic!("only support 1 field");
        }
        if matches!(ele_type, EleType::Text) {
            if name.is_some() {
                panic!("should omit the `name`");
            }
        } else if name.is_none() {
            panic!("should have name")
        }
        let field = &v.fields.iter().next();
        let ty = field.map(|t| &t.ty);
        let ident = &v.ident;
        EnumVariant {
            name,
            ty,
            ident,
            ele_type,
        }
    }
}

/// Specify where this field is in the xml.
pub enum EleType {
    Attr,
    Child,
    Text,
    ///
    /// ```
    /// struct Font {
    ///     bold: bool,
    ///     italic: bool,
    /// }
    /// ```
    /// In the xml, it is like
    /// <font>
    ///     <b/>
    ///     <i/>
    /// </font>
    /// In this case, </b> indicates the field *bold* is true and <i/> indicates *italic* is true.
    SelfClosedChild,
    Untag,
    Any,
    XmlNs,
}

#[allow(dead_code)]
pub enum Derive {
    Serialize,
    Deserialize,
}

fn get_xmlserde_meta_items(attr: &syn::Attribute) -> Result<Vec<syn::Meta>, ()> {
    if attr.path() != RAXB {
        return Ok(Vec::new());
    }

    match attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated) {
        Ok(meta) => Ok(meta.into_iter().collect()),
        Err(_) => Err(()),
    }
}

fn get_lit_byte_str(expr: &syn::Expr) -> Result<&syn::LitByteStr, ()> {
    if let syn::Expr::Lit(lit) = expr {
        if let syn::Lit::ByteStr(l) = &lit.lit {
            return Ok(l);
        }
    }
    Err(())
}

fn get_lit_str(lit: &syn::Expr) -> Result<&syn::LitStr, ()> {
    if let syn::Expr::Lit(lit) = lit {
        if let syn::Lit::Str(l) = &lit.lit {
            return Ok(l);
        }
    }
    Err(())
}

fn get_generics(t: &syn::Type) -> Generic {
    match t {
        syn::Type::Path(p) => {
            let path = &p.path;
            match path.segments.last() {
                Some(seg) => {
                    if seg.ident == "Vec" {
                        match &seg.arguments {
                            syn::PathArguments::AngleBracketed(a) => {
                                let args = &a.args;
                                if args.len() != 1 {
                                    Generic::None
                                } else if let Some(syn::GenericArgument::Type(t)) = args.first() {
                                    Generic::Vec(t)
                                } else {
                                    Generic::None
                                }
                            }
                            _ => Generic::None,
                        }
                    } else if seg.ident == "Option" {
                        match &seg.arguments {
                            syn::PathArguments::AngleBracketed(a) => {
                                let args = &a.args;
                                if args.len() != 1 {
                                    Generic::None
                                } else if let Some(syn::GenericArgument::Type(t)) = args.first() {
                                    Generic::Opt(t)
                                } else {
                                    Generic::None
                                }
                            }
                            _ => Generic::None,
                        }
                    } else {
                        Generic::None
                    }
                }
                None => Generic::None,
            }
        }
        _ => Generic::None,
    }
}

pub enum Generic<'a> {
    Vec(&'a syn::Type),
    Opt(&'a syn::Type),
    None,
}

impl<'a> Generic<'a> {
    pub fn get_opt(&self) -> Option<&syn::Type> {
        match self {
            Generic::Opt(v) => Some(v),
            _ => None,
        }
    }
}

use quote::quote;
use syn::{ExprArray, Ident, LitByteStr, Type};

use crate::{
    container::{Container, EleType, Generic, StructField},
    utils::get_built_in_type,
};

#[derive(Debug)]
enum TextValueType {
    XmlValue,
    String,
    Parse,
}

fn get_text_value_type(original_type: &Type) -> TextValueType {
    match get_built_in_type(original_type) {
        crate::container::BuiltInType::String => TextValueType::String,
        crate::container::BuiltInType::XmlValue => TextValueType::XmlValue,
        _ => TextValueType::Parse,
    }
}

enum TextPathValue<'a> {
    String(String),
    Expr(&'a ExprArray)
}

struct TextPath<'a> {
    ident: &'a syn::Ident,
    ident_value: TextPathValue<'a>,
    path_ident: syn::Ident,
    ty: TextValueType,
    sf: &'a StructField<'a>,
}

fn filter_text_paths<'a>(container: &'a Container) -> Vec<TextPath<'a>> {
    container
        .struct_fields
        .iter()
        .filter_map(|f| {
            let original_type = match f.generic {
                Generic::Vec(ty) => ty,
                Generic::Opt(ty) => ty,
                Generic::None => &f.original.ty,
            };
            let ty = get_text_value_type(original_type);
            if matches!(f.ty, EleType::Text) {
                let ident = &f.original.ident;
                if let Some(ident) = ident.as_ref() {
                    let ident_str = ident.to_string();
                    let path_ident: syn::Ident =
                        syn::parse_str(&format!("{}_path", ident_str)).unwrap();
                    let ident_value = if let Some(path) = &f.path {
                        TextPathValue::Expr(path)
                    } else {
                        TextPathValue::String(ident_str)
                    };

                    return Some(TextPath {
                        ident,
                        ident_value,
                        path_ident,
                        ty,
                        sf: f,
                    });
                }
            }
            None
        })
        .collect::<Vec<_>>()
}

fn get_assignment(ident: &Ident, generic: &Generic<'_>, val: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    match generic {
        Generic::Vec(_) => quote! {
            #ident.push(#val);
        },
        Generic::Opt(_) => quote! {
            #ident = Some(#val);
        },
        Generic::None => quote! {
            #ident = Some(#val);
        },
    }
}

fn get_first_ty_segment<'a>(sf: &StructField<'a>) -> Option<&'a Ident> {
    let ty = match sf.generic {
        Generic::Vec(ty) => ty,
        Generic::Opt(ty) => ty,
        Generic::None => &sf.original.ty,
    };
    if let Type::Path(p) = ty {
        if let Some(ty) = p.path.segments.first() {
            return Some(&ty.ident);
        }
    }
    None
}

pub fn impl_block(container: Container) -> proc_macro2::TokenStream {
    let ident = &container.original.ident;
    let mut first_lt = None;
    for lt in container.original.generics.lifetimes() {
        first_lt = Some(lt.lifetime.clone());
        break;
    }
    let fields_init = container
        .struct_fields
        .iter()
        .filter_map(|f| {
            let ty = &f.original.ty;
            let ident = &f.original.ident;
            if let Some(ident) = ident.as_ref() {
                return Some(match f.generic {
                    Generic::Vec(_) => quote! {
                        let mut #ident : #ty = vec![];
                    },
                    Generic::Opt(_) => quote! {
                        let mut #ident : #ty = None;
                    },
                    Generic::None => quote! {
                        let mut #ident : Option<#ty> = None;
                    },
                });
            }
            None
        })
        .collect::<Vec<_>>();
    let text_paths = filter_text_paths(&container);
    let paths_init = text_paths
        .iter()
        .map(|p| {
            let path_ident = &p.path_ident;
            match &p.ident_value {
                TextPathValue::String(ident_str) => quote! {
                    let #path_ident = &[#ident_str][..];
                },
                TextPathValue::Expr(array) => quote! {
                    let #path_ident = &#array[..];
                },
            }
        })
        .collect::<Vec<_>>();
    let text_branches = text_paths
        .iter()
        .map(|f| {
            let ident = f.ident;
            let path_ident = &f.path_ident;
            let val = match f.ty {
                TextValueType::XmlValue => quote! { bytes_text.unescape()? },
                TextValueType::String =>   quote! { bytes_text.unescape()? },
                TextValueType::Parse =>    quote! { bytes_text.unescape()?.parse()? },
            };
            let assignment = get_assignment(ident, &f.sf.generic, val);
            quote! {
                if p == #path_ident {
                    #assignment
                }
            }
        })
        .collect::<Vec<_>>();
    let child_branches = container
        .struct_fields
        .iter()
        .filter(|f| matches!(f.ty, EleType::Child) && f.original.ident.is_some())
        .filter_map(|f| {
            let ident = f.original.ident.as_ref().unwrap();
            let name = f.name.clone().unwrap_or_else(|| syn::parse_str(&format!("b\"{}\"", ident.to_string())).unwrap());
            if  let Some(ty) = get_first_ty_segment(f) {
                let val = quote!{
                    #ty::xml_borrow(reader, Some(bytes_start))?
                };
                let assignment = get_assignment(ident, &f.generic, val);
                return Some(quote! {
                    #name => {
                        #assignment
                    }
                });
            }
            None
        })
        .collect::<Vec<_>>();
    let sfc_branches = container
        .struct_fields
        .iter()
        .filter(|f| matches!(f.ty, EleType::SelfClosedChild) && f.original.ident.is_some())
        .filter_map(|f| {
            let ident = f.original.ident.as_ref().unwrap();
            let name = f.name.clone().unwrap_or_else(|| syn::parse_str(&format!("b\"{}\"", ident.to_string())).unwrap());
            if  let Some(ty) = get_first_ty_segment(f) {
                let t = get_built_in_type(&f.original.ty);
                let val = if t.is_bool() {
                    quote!{
                        true
                    }
                } else {
                    quote!{
                        #ty::xml_borrow(reader, Some(bytes_start))?
                    }
                };
                let assignment = get_assignment(ident, &f.generic, val);
                return Some(quote! {
                    #name => {
                        #assignment
                    }
                });
            }
            None
        })
        .collect::<Vec<_>>();
    let attr_branches = container
        .struct_fields
        .iter()
        .filter(|f| matches!(f.ty, EleType::Attr) && f.original.ident.is_some())
        .map(|f| {
            let ident = f.original.ident.as_ref().unwrap();
            let name = f.name.clone().unwrap_or_else(|| syn::parse_str(&format!("b\"{}\"", ident.to_string())).unwrap());
            let ty = get_text_value_type(&f.original.ty);
            let val = match ty {
                TextValueType::XmlValue => quote! { XmlValue::Owned(attr.to_string()) },
                TextValueType::String => quote! { attr.to_string() },
                TextValueType::Parse => quote! { attr.to_string().parse()? },
            };
            let assignment = get_assignment(ident, &f.generic, val);
            quote! {
                #name => {
                    let attr = attr.to_owned().decode_and_unescape_value(reader.decoder())?;
                    #assignment
                }
            }
        })
        .collect::<Vec<_>>();
    let results = container
        .struct_fields
        .iter()
        .filter_map(|f| {
            let ident = &f.original.ident;
            if let Some(ident) = ident.as_ref() {
                if f.default {
                    return Some(quote! {
                        #ident: #ident.unwrap_or_default()
                    })
                }
                if matches!(f.generic, Generic::None) {
                    let bident: LitByteStr =
                        syn::parse_str(&format!("b\"{}\"", ident.to_string())).unwrap();
                    let error = if matches!(f.ty, EleType::Attr) {
                        quote! {
                            XmlBorrowError::MissingAttribute(_raxb::ty::S(#bident))
                        }
                    } else {
                        quote! {
                            XmlBorrowError::MissingElement(_raxb::ty::S(#bident))
                        }
                    };
                    return Some(quote! {
                        #ident: #ident.ok_or_else(|| #error)?
                    });
                }
                return Some(quote! {
                    #ident
                });
            }
            None
        })
        .collect::<Vec<_>>();
    let (impl_generics, type_generics, where_clause) = container.original.generics.split_for_impl();
    quote! {
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute, clippy::manual_flatten, clippy::single_match)]
            extern crate raxb as _raxb;
            use _raxb::{
                zde::{XmlBorrow, XmlBorrowError, XmlBorrowResult, Pointer},
                quick_xml::{
                    events::{attributes::Attributes, Event, BytesStart},
                    name::ResolveResult,
                    Reader,
                },
                ty::{XmlTag, XmlTargetNs, S},
            };
            #[automatically_derived]
            impl #impl_generics XmlBorrow <#first_lt> for #ident #type_generics #where_clause {
                fn xml_borrow(
                    reader: &mut Reader<&#first_lt [u8]>,
                    bytes_start: Option<BytesStart<#first_lt>>,
                ) -> XmlBorrowResult<Self> {
                    #(#fields_init)*
                    #(#paths_init)*

                    if let Some(bytes_start) = bytes_start.as_ref() {
                        let attrs = bytes_start.attributes();
                        for attr in attrs {
                            let attr = attr?;
                            match attr.key.local_name().as_ref() {
                                #(#attr_branches,)*
                                _ => {}
                            }
                        }
                    }
                    let mut p = Pointer::new();
                    loop {
                        let ev = reader.read_event()?;
                        match ev {
                            Event::Start(bytes_start) => {
                                match bytes_start.local_name().as_ref() {
                                    #(#child_branches,)*
                                    _ => {
                                        p.visit(bytes_start);
                                    }
                                }
                            },
                            Event::Empty(bytes_start) => {
                                match bytes_start.local_name().as_ref() {
                                    #(#sfc_branches,)*
                                    _ => {}
                                }
                            },
                            Event::Text(bytes_text) => {
                                #(#text_branches)*
                            }
                            Event::End(bytes_end) => {
                                if let Some(bytes_start) = bytes_start.as_ref() {
                                    let end = bytes_start.to_end();
                                    if bytes_end == end {
                                        break;
                                    }
                                }
                                p.leave();
                            },
                            Event::Eof => break,
                            _ => {}
                        }
                    }

                    Ok(Self {
                        #(#results,)*
                    })
                }
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use syn::DeriveInput;

    use crate::container::{self, Container};
    fn pretty_print_item(item: proc_macro2::TokenStream) -> String {
        let item = syn::parse2(item).unwrap();
        let file = syn::File {
            attrs: vec![],
            items: vec![item],
            shebang: None,
        };
        prettyplease::unparse(&file)
    }
    #[test]
    fn test_fields_expansion() -> anyhow::Result<()> {
        let code = r#"
        pub struct A<'a> {
            #[xml(ty = "attr")]
            pub name: XmlValue<'a>,
            #[xml(default, ty = "sfc")]
            pub b: bool,
            pub b2: bool,
            pub f1: Vec<i32>,
            #[xml(path = ["c", "x", "y"])]
            pub c: XmlValue<'a>,
            pub d: XmlValue<'a>,
            #[xml(ty = "child")]
            pub e: Option<D<'a>>,
        }
        "#;
        let t =
            proc_macro2::TokenStream::from_str(&code).map_err(|err| anyhow::anyhow!("{err:#?}"))?;
        let input = syn::parse2::<DeriveInput>(t)?;
        let container = Container::from_ast(&input, container::Derive::Deserialize);
        container.validate();
        eprintln!("{}", pretty_print_item(super::impl_block(container)));

        Ok(())
    }
}

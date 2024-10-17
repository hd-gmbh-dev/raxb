use quote::quote;
use syn::{LitByteStr, Type};

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

struct TextPath<'a> {
    ident: &'a syn::Ident,
    ident_str: String,
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
                    return Some(TextPath {
                        ident,
                        ident_str,
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
                if matches!(f.generic, Generic::Vec(_)) {
                    return Some(quote! {
                        let mut #ident : Vec<#ty> = vec![];
                    });
                } else {
                    return Some(quote! {
                        let mut #ident : Option<#ty> = None;
                    });
                }
            }
            None
        })
        .collect::<Vec<_>>();
    let text_paths = filter_text_paths(&container);
    let paths_init = text_paths
        .iter()
        .map(|p| {
            let path_ident = &p.path_ident;
            let ident_str = &p.ident_str;
            quote! {
                let #path_ident = &[#ident_str][..];
            }
        })
        .collect::<Vec<_>>();
    let text_branches = text_paths
        .iter()
        .map(|f| {
            let ident = f.ident;
            let path_ident = &f.path_ident;
            let val = match f.ty {
                TextValueType::XmlValue => quote! { Some(bytes_text.unescape()?) },
                TextValueType::String => quote! { Some(bytes_text.unescape()?) },
                TextValueType::Parse => quote! { Some(bytes_text.unescape()?.parse()?) },
            };
            let assignment = match f.sf.generic {
                Generic::Vec(_) => quote! {
                    #ident.push(#val);
                },
                Generic::Opt(_) => quote! {
                    #ident = Some(#val);
                },
                Generic::None => quote! {
                    #ident = #val;
                },
            };
            quote! {
                if p == #path_ident {
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
                if matches!(f.generic, Generic::Opt(_)) {}
                if matches!(f.generic, Generic::None) {
                    let bident: LitByteStr =
                        syn::parse_str(&format!("b\"{}\"", ident.to_string())).unwrap();
                    return Some(quote! {
                        #ident: #ident.ok_or(XmlBorrowError::MissingElement(_raxb::ty::S(#bident)))?
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

                    let mut p = Pointer::new();
                    loop {
                        let ev = reader.read_event()?;
                        match ev {
                            Event::Start(bytes_start) => {
                                p.visit(bytes_start);
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
            #[xml(ty = "sfc")]
            pub b: bool,
            pub b2: bool,
            pub f1: Vec<i32>,
            pub c: XmlValue<'a>,
            pub d: XmlValue<'a>,
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

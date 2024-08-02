use proc_macro2::Span;
use quote::quote;
use syn::LitByteStr;

use crate::{
    container::{Container, EnumVariant},
    utils::{create_tns_impl, get_built_in_type, trace},
};

fn create_variant(
    ident: &syn::Ident,
    variant: &EnumVariant,
    empty: bool,
) -> Option<proc_macro2::TokenStream> {
    let name = variant.name.as_ref();
    let variant_ident = variant.ident;
    let ty = variant.ty.as_ref();
    if let Some((name, ty)) = name.zip(ty) {
        let built_in_type = get_built_in_type(ty);
        let assignment = if built_in_type.is_bool() || built_in_type.is_number() {
            quote! {
                let value = if let (_, Event::Text(t)) = reader.read_resolved_event_into(&mut buf)? {
                    let str_value = t.unescape()?;
                    let value : #ty = str_value.parse().unwrap_or_default();
                    result = Some(#ident::#variant_ident(value));
                };
            }
        } else if built_in_type.is_string() {
            quote! {
                let value = if let (_, Event::Text(t)) = reader.read_resolved_event_into(&mut buf)? {
                    let str_value = t.unescape()?;
                    let value = str_value.to_string();
                    result = Some(#ident::#variant_ident(value));
                };
            }
        } else {
            quote! {
                let value = #ty::xml_deserialize(
                    reader,
                    target_ns,
                    #name,
                    e.attributes(),
                    #empty,
                )?;
                result = Some(#ident::#variant_ident(value));
            }
        };
        return Some(quote! {
            #name => {
                #assignment
                break;
            },
        });
    } else if let Some(name) = name {
        return Some(quote! {
            #name => {
                result = Some(#ident::#variant_ident);
                break;
            },
        });
    }
    None
}

pub fn impl_block(container: Container) -> proc_macro2::TokenStream {
    let ident = &container.original.ident;
    let ident_str = ident.to_string();
    let tns_impl = create_tns_impl(&container);
    let variants: Vec<proc_macro2::TokenStream> = container
        .enum_variants
        .iter()
        .filter_map(|variant| create_variant(&container.original.ident, variant, false))
        .collect();
    let empty_variants: Vec<proc_macro2::TokenStream> = container
        .enum_variants
        .iter()
        .filter_map(|variant| create_variant(&container.original.ident, variant, true))
        .collect();

    let qualified_variants = variants.iter();
    let qualified_empty_variants = empty_variants.iter();
    let unqualified_variants = variants.iter();
    let unqualified_empty_variants = empty_variants.iter();

    let enum_err = container
        .enum_variants
        .iter()
        .filter_map(|v| v.name.as_ref())
        .map(|v| format!("'{}'", String::from_utf8(v.value()).unwrap()))
        .collect::<Vec<String>>()
        .join("|");
    let enum_err = LitByteStr::new(enum_err.as_bytes(), Span::call_site());
    let (impl_generics, type_generics, where_clause) = container.original.generics.split_for_impl();
    let trace_event = |event_type: &'static str| {
        trace(quote! {
            if tag.is_empty() {
                if target_ns.is_empty() {
                    debug!("{} enum '{}'", #event_type, #ident_str);
                } else {
                    debug!("{} enum '{}' with namespace '{}'", #event_type, #ident_str, std::str::from_utf8(target_ns).unwrap());
                }
            } else {
                if target_ns.is_empty() {
                    debug!("{} enum '{}' with tag '{}'", #event_type, #ident_str, std::str::from_utf8(tag).unwrap());
                } else {
                    debug!("{} enum '{}' with tag '{}' and namespace '{}'", #event_type, #ident_str, std::str::from_utf8(tag).unwrap(), std::str::from_utf8(target_ns).unwrap());
                }
            }
        })
    };
    let trace_enter_enum = trace_event("Enter");
    let trace_leave_enum = trace_event("Leave");
    quote! {
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute, clippy::manual_flatten, clippy::single_match)]
            extern crate raxb as _raxb;

            use _raxb::{
                de::{XmlDeserialize, XmlDeserializeError, XmlDeserializeResult},
                quick_xml::{
                    events::{attributes::Attributes, Event},
                    name::ResolveResult,
                    NsReader,
                },
                ty::{XmlTag, XmlTargetNs, S},
            };
            #[automatically_derived]
            impl #impl_generics _raxb::de::XmlDeserialize for #ident #type_generics #where_clause {
                fn xml_deserialize<R: std::io::BufRead>(
                    reader: &mut _raxb::quick_xml::NsReader<R>,
                    target_ns: _raxb::ty::XmlTag,
                    tag: _raxb::ty::XmlTargetNs,
                    attributes: _raxb::quick_xml::events::attributes::Attributes,
                    is_empty: bool,
                ) -> _raxb::de::XmlDeserializeResult<Self> {
                    let target_ns = Self::target_ns().unwrap_or(target_ns);
                    #trace_enter_enum
                    let mut result = Option::<#ident>::None;
                    let mut buf = Vec::<u8>::new();
                    loop {
                        match reader.read_resolved_event_into(&mut buf)? {
                            (ResolveResult::Unbound, Event::Start(e)) => {
                                match e.local_name().as_ref() {
                                    #(#unqualified_variants)*
                                    _ => {
                                        let mut buf = Vec::<u8>::new();
                                        reader.read_to_end_into(e.name(), &mut buf)?;
                                    }
                                }
                            }
                            (ResolveResult::Unbound, Event::Empty(e)) => {
                                match e.local_name().as_ref() {
                                    #(#unqualified_empty_variants)*
                                    _ => {}
                                }
                            }
                            (ResolveResult::Bound(ns), Event::Start(e)) => if ns.as_ref() == target_ns {
                                match e.local_name().as_ref() {
                                    #(#qualified_variants)*
                                    _ => {
                                        let mut buf = Vec::<u8>::new();
                                        reader.read_to_end_into(e.name(), &mut buf)?;
                                    }
                                }
                            }
                            (ResolveResult::Bound(ns), Event::Empty(e)) => if ns.as_ref() == target_ns {
                                match e.local_name().as_ref() {
                                    #(#qualified_empty_variants)*
                                    _ => {}
                                }
                            }
                            (_, Event::Eof) => {
                                break;
                            }
                            _ => {}
                        }
                    }
                    if result.is_some() {
                        if tag.is_empty() {
                            #trace_leave_enum
                        } else {
                            loop {
                                match reader.read_resolved_event_into(&mut buf)? {
                                    (ResolveResult::Bound(ns), Event::End(e)) => if ns.as_ref() == target_ns && e.local_name().as_ref() == tag {
                                        #trace_leave_enum
                                        break;
                                    },
                                    (ResolveResult::Unbound, Event::End(e)) => if e.local_name().as_ref() == tag {
                                        #trace_leave_enum
                                        break;
                                    },
                                    (_, Event::Eof) => {
                                        break;
                                    }
                                    #[cfg(not(feature="trace"))]
                                    _ => {},
                                    #[cfg(feature="trace")]
                                    ev => {
                                        _raxb::tracing::warn!("Unexpected Event: {ev:#?}");
                                    },
                                }
                            }
                        }
                    }
                    result.ok_or(XmlDeserializeError::MissingVariant(S(#enum_err)))
                }

                #tns_impl

                fn is_enum() -> bool {
                    true
                }
            }
        };
    }
}

use std::str::FromStr;

use quote::quote;

use crate::container::{BuiltInType, Container, EleType, FieldsSummary, Generic};

pub fn init(fields: &FieldsSummary) -> proc_macro2::TokenStream {
    let v = fields.children.iter().map(super::utils::create_ident);
    quote! {
        #(#v)*
    }
}

pub fn create_assignments(container: &Container) -> proc_macro2::TokenStream {
    let mut child_branches = Vec::new();
    let mut sfc_branches = Vec::new();

    for f in container
        .struct_fields
        .iter()
        .filter(|sf| matches!(sf.ty, EleType::Child | EleType::SelfClosedChild))
    {
        if let Some(tag) = f.name.as_ref() {
            let ident = f.original.ident.as_ref().unwrap();
            let ty = &f.original.ty;
            let (ty, is_array) = match f.generic {
                Generic::Vec(v) => (v, true),
                Generic::Opt(opt) => (opt, false),
                Generic::None => (ty, false),
            };
            if matches!(f.ty, EleType::Child) {
                let deserialize_value = create_deserialize_value(tag, ty, ident, is_array);
                child_branches.push(quote! {
                    #tag => {
                        #deserialize_value
                    }
                });
            }
            if matches!(f.ty, EleType::Child) {
                let deserialize_value_sfc = create_deserialize_value_sfc(tag, ty, ident, is_array);
                sfc_branches.push(quote! {
                    #tag => {
                        #deserialize_value_sfc
                    }
                });
            }
        }
    }

    let has_children = !child_branches.is_empty();
    let has_sfcs = !child_branches.is_empty();

    let child_branches = child_branches.into_iter();
    let sfc_branches = sfc_branches.into_iter();

    if has_children || has_sfcs {
        quote! {
            use _raxb::quick_xml::{events::Event, name::ResolveResult};
            let mut buf = Vec::<u8>::new();

            loop {
                match reader.read_resolved_event_into(&mut buf)? {
                    (ResolveResult::Unbound, Event::Start(ev)) => {
                        match ev.local_name().as_ref() {
                            #(#child_branches,)*
                            _ => {
                                let mut buffer: Vec<u8> = Vec::<u8>::new();
                                reader.read_to_end_into(ev.name(), &mut buffer)?;
                            },
                        }
                    },
                    (ResolveResult::Unbound, Event::Empty(ev)) => {
                        match ev.local_name().as_ref() {
                            #(#sfc_branches,)*
                            _ => {},
                        }
                    },
                    (ResolveResult::Unbound, Event::End(e)) if e.local_name().as_ref() == tag => {
                        break;
                    },
                    (ResolveResult::Unbound, Event::Eof) => {
                        break;
                    },
                    _ => {},
                }
            }
        }
    } else {
        quote! {}
    }
}

fn create_deserialize_value_sfc(
    tag: &syn::LitByteStr,
    ty: &syn::Type,
    ident: &syn::Ident,
    is_array: bool,
) -> proc_macro2::TokenStream {
    let assignment = if is_array {
        quote! {
            #ident.push(value);
        }
    } else {
        quote! {
            #ident = Some(value);
        }
    };
    if let syn::Type::Path(p) = ty {
        if let Some(ident) = p.path.get_ident() {
            let built_in_ty: BuiltInType =
                BuiltInType::from_str(&format!("{ident}")).unwrap_or_default();
            if built_in_ty.is_bool() {
                return quote! {
                    let value = true;
                    #assignment
                };
            } else if built_in_ty.is_unknown() {
                return quote! {
                    let value = #ty::xml_deserialize(reader, &[], #tag, ev.attributes(), true)?;
                    #assignment
                };
            }
        }
    }
    quote! {}
}

fn create_deserialize_value(
    tag: &syn::LitByteStr,
    ty: &syn::Type,
    ident: &syn::Ident,
    is_array: bool,
) -> proc_macro2::TokenStream {
    let assignment = if is_array {
        quote! {
            #ident.push(value);
        }
    } else {
        quote! {
            #ident = Some(value);
        }
    };
    if let syn::Type::Path(p) = ty {
        if let Some(ident) = p.path.get_ident() {
            let built_in_ty: BuiltInType =
                BuiltInType::from_str(&format!("{ident}")).unwrap_or_default();
            if built_in_ty.is_string() {
                return quote! {
                    let mut buffer: Vec<u8> = Vec::<u8>::new();
                    if let (ResolveResult::Unbound, Event::Text(t)) = reader.read_resolved_event_into(&mut buffer)? {
                        let value = t.unescape()?.to_string();
                        #assignment
                    }
                };
            } else if built_in_ty.is_bool() || built_in_ty.is_number() {
                return quote! {
                    let mut buffer: Vec<u8> = Vec::<u8>::new();
                    if let (ResolveResult::Unbound, Event::Text(t)) = reader.read_resolved_event_into(&mut buffer)? {
                        let str_value = t.unescape()?;
                        let value : #ty = str_value.parse()?;
                        #assignment
                    }
                };
            } else {
                return quote! {
                    let value = #ty::xml_deserialize(reader, &[], #tag, ev.attributes(), false)?;
                    #assignment
                };
            }
        }
    }
    quote! {}
}

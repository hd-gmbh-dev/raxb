use std::str::FromStr;

use quote::quote;

use crate::container::{BuiltInType, Container, EleType, FieldsSummary, Generic};

pub fn init(fields: &FieldsSummary) -> proc_macro2::TokenStream {
    let v = fields.children.iter().map(crate::utils::create_ident);
    let s = fields
        .self_closed_children
        .iter()
        .map(crate::utils::create_ident);
    quote! {
        #(#v)*
        #(#s)*
    }
}

pub fn create_assignments(container: &Container) -> proc_macro2::TokenStream {
    let mut qualified_child_branches = Vec::<proc_macro2::TokenStream>::new();
    let mut qualified_sfc_branches = Vec::<proc_macro2::TokenStream>::new();
    let mut unqualified_child_branches = Vec::<proc_macro2::TokenStream>::new();
    let mut unqualified_sfc_branches = Vec::<proc_macro2::TokenStream>::new();

    for f in container
        .struct_fields
        .iter()
        .filter(|sf| matches!(sf.ty, EleType::Child | EleType::SelfClosedChild))
    {
        let is_qualified = f.ns.is_some();
        if let Some(tag) = f.name.as_ref() {
            let ident = f.original.ident.as_ref().unwrap();
            let ty = &f.original.ty;
            let (ty, is_array) = match f.generic {
                Generic::Vec(v) => (v, true),
                Generic::Opt(opt) => (opt, false),
                Generic::None => (ty, false),
            };
            if matches!(f.ty, EleType::Child) {
                if is_qualified {
                    let deserialize_value = create_deserialize_value(tag, ty, ident, is_array);
                    qualified_child_branches.push(quote! {
                        #tag => {
                            #deserialize_value
                        }
                    });
                } else {
                    let deserialize_value = create_deserialize_value(tag, ty, ident, is_array);
                    unqualified_child_branches.push(quote! {
                        #tag => {
                            #deserialize_value
                        }
                    });
                }
            }
            if matches!(f.ty, EleType::SelfClosedChild) {
                if is_qualified {
                    let deserialize_value_sfc =
                        create_deserialize_value_sfc(tag, ty, ident, is_array);
                    qualified_sfc_branches.push(quote! {
                        #tag => {
                            #deserialize_value_sfc
                        }
                    });
                } else {
                    let deserialize_value_sfc =
                        create_deserialize_value_sfc(tag, ty, ident, is_array);
                    unqualified_sfc_branches.push(quote! {
                        #tag => {
                            #deserialize_value_sfc
                        }
                    });
                }
            }
        }
    }

    let has_unqualified_children = !unqualified_child_branches.is_empty();
    let has_unqualified_sfcs = !unqualified_child_branches.is_empty();

    let has_qualified_children = !qualified_child_branches.is_empty();
    let has_qualified_sfcs = !qualified_child_branches.is_empty();

    let unqualified_child_branches = unqualified_child_branches.into_iter();
    let unqualified_sfc_branches = unqualified_sfc_branches.into_iter();
    let qualified_child_branches = qualified_child_branches.into_iter();
    let qualified_sfc_branches = qualified_sfc_branches.into_iter();
    let qualified_child_branch = if has_qualified_children {
        quote! {
            (ResolveResult::Bound(ns), Event::Start(ev)) => {
                match ev.local_name().as_ref() {
                    #(#qualified_child_branches,)*
                    _ => {
                        let mut buffer: Vec<u8> = Vec::<u8>::new();
                        reader.read_to_end_into(ev.name(), &mut buffer)?;
                    },
                }
            },
        }
    } else {
        quote! {}
    };
    let qualified_sfc_branch = if has_qualified_children {
        quote! {
            (ResolveResult::Bound(ns), Event::Empty(ev)) => {
                match ev.local_name().as_ref() {
                    #(#qualified_sfc_branches,)*
                    _ => {},
                }
            },
        }
    } else {
        quote! {}
    };

    let unqualified_child_branch = if has_unqualified_children {
        quote! {
            (ResolveResult::Unbound, Event::Start(ev)) => {
                match ev.local_name().as_ref() {
                    #(#unqualified_child_branches,)*
                    _ => {
                        let mut buffer: Vec<u8> = Vec::<u8>::new();
                        reader.read_to_end_into(ev.name(), &mut buffer)?;
                    },
                }
            },
        }
    } else {
        quote! {}
    };

    let unqualified_sfc_branch = if has_unqualified_children {
        quote! {
            (ResolveResult::Unbound, Event::Empty(ev)) => {
                match ev.local_name().as_ref() {
                    #(#unqualified_sfc_branches,)*
                    _ => {},
                }
            },
        }
    } else {
        quote! {}
    };

    let end_branch = if container.tns.is_some() {
        let tns = &container.tns.as_ref().unwrap().1;
        quote! {
            (ResolveResult::Bound(ns), Event::End(e)) if e.local_name().as_ref() == tag && ns.as_ref() == #tns =>  {
                break;
            },
        }
    } else {
        quote! {
            (ResolveResult::Unbound, Event::End(e)) if e.local_name().as_ref() == tag => {
                break;
            },
        }
    };

    if has_unqualified_children
        || has_unqualified_sfcs
        || has_qualified_children
        || has_qualified_sfcs
    {
        quote! {
            use _raxb::quick_xml::{events::Event, name::ResolveResult};
            let mut buf = Vec::<u8>::new();

            loop {
                match reader.read_resolved_event_into(&mut buf)? {
                    #qualified_child_branch
                    #qualified_sfc_branch
                    #unqualified_child_branch
                    #unqualified_sfc_branch
                    #end_branch
                    (_, Event::Eof) => {
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
                    if let (_, Event::Text(t)) = reader.read_resolved_event_into(&mut buffer)? {
                        let value = t.unescape()?.to_string();
                        #assignment
                    }
                };
            } else if built_in_ty.is_bool() || built_in_ty.is_number() {
                return quote! {
                    let mut buffer: Vec<u8> = Vec::<u8>::new();
                    if let (_, Event::Text(t)) = reader.read_resolved_event_into(&mut buffer)? {
                        let str_value = t.unescape()?;
                        let value : #ty = str_value.parse()?;
                        #assignment
                    }
                };
            } else {
                return quote! {
                    let value = #ty::xml_deserialize(reader, target_ns, #tag, ev.attributes(), false)?;
                    #assignment
                };
            }
        }
    }
    quote! {}
}

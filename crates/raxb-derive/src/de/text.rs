use std::str::FromStr;

use quote::quote;

use crate::container::{BuiltInType, FieldsSummary, Generic, StructField};

pub fn init(fields: &FieldsSummary) -> proc_macro2::TokenStream {
    if let Some(f) = fields.text.as_ref() {
        return crate::utils::create_ident(f);
    }
    quote! {}
}

fn create_assing_value(
    f: &StructField,
) -> (proc_macro2::TokenStream, Option<proc_macro2::TokenStream>) {
    let ident = f.original.ident.as_ref().unwrap();
    let ty = &f.original.ty;
    let (ty, is_array) = match f.generic {
        Generic::Vec(ty) => (ty, true),
        Generic::Opt(opt) => (opt, false),
        Generic::None => (ty, false),
    };
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
        if let Some(ty_ident) = p.path.get_ident() {
            let built_in_ty: BuiltInType =
                BuiltInType::from_str(&format!("{ty_ident}")).unwrap_or_default();
            return if built_in_ty.is_string() {
                (
                    quote! {
                        let value = ev.unescape()?.to_string();
                        #assignment
                    },
                    Some(quote! {
                        let value = String::new();
                        #assignment
                    }),
                )
            } else {
                (
                    quote! {
                        let str_value = ev.unescape()?;
                        let value : #ty = str_value.trim().parse()?;
                        #assignment
                    },
                    None,
                )
            };
        }
    }
    (quote! {}, None)
}

pub fn create_assignments(f: &StructField) -> proc_macro2::TokenStream {
    let (assign_value, assign_empty_value) = create_assing_value(f);
    quote! {
        if is_empty {
            #assign_empty_value
        } else {
            let mut buf = Vec::<u8>::new();
            loop {
                match reader.read_resolved_event_into(&mut buf)? {
                    (_, Event::Text(ev)) => {
                        #assign_value
                    },
                    (_, Event::Start(ev)) => {
                        let mut buffer: Vec<u8> = Vec::<u8>::new();
                        reader.read_to_end_into(ev.name(), &mut buffer)?;
                    },
                    (_, Event::Empty(ev)) => {},
                    (_, Event::End(e)) if e.local_name().as_ref() == tag => {
                        break;
                    },
                    (_, Event::Eof) => {
                        break;
                    },
                    _ => {},
                }
            }
        }
    }
}

use std::str::FromStr;

use quote::quote;

use crate::container::{BuiltInType, FieldsSummary};

pub fn init(fields: &FieldsSummary) -> proc_macro2::TokenStream {
    let v = fields.attrs.iter().map(|f| {
        let ident = f.original.ident.as_ref().unwrap();
        let ty = &f.original.ty;
        if let Some(opt) = f.generic.get_opt() {
            quote! {
                let mut #ident = Option::<#opt>::None;
            }
        } else {
            quote! {
                let mut #ident = Option::<#ty>::None;
            }
        }
    });
    quote! {
        #(#v)*
    }
}

pub fn create_assignments(fields: &FieldsSummary) -> proc_macro2::TokenStream {
    if fields.attrs.is_empty() {
        return quote! {};
    }
    let attrs = fields.attrs.iter().filter_map(|f| {
        f.name.as_ref()?;
        let ident = f.original.ident.as_ref().unwrap();
        let name = f.name.as_ref().unwrap();
        let ty = &f.original.ty;
        let ty = if let Some(opt) = f.generic.get_opt() {
            opt
        } else {
            ty
        };
        if let syn::Type::Path(p) = ty {
            if let Some(ty_ident) = p.path.get_ident() {
                let built_in_type =
                    BuiltInType::from_str(&format!("{ty_ident}")).unwrap_or_default();
                if built_in_type.is_string() {
                    return Some(quote! {
                        #name => {
                            let value_str = String::from_utf8(attr.value.to_vec())?;
                            let value = _raxb::quick_xml::escape::unescape(&value_str)?;
                            #ident = Some(value.to_string());
                        }
                    });
                } else {
                    return Some(quote! {
                        #name => {
                            let value_str = String::from_utf8(attr.value.to_vec())?;
                            let value = _raxb::quick_xml::escape::unescape(&value_str)?;
                            #ident = Some(value.trim().parse().unwrap_or_default());
                        }
                    });
                }
            }
        }
        None
    });
    quote! {
        for attr in attributes.flatten() {
            match attr.key.local_name().as_ref() {
                #(#attrs)*
                _ => {}
            }
        }
    }
}

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
    if let syn::Type::Path(p) = ty
        && let Some(ty_ident) = p.path.get_ident()
    {
        let built_in_ty: BuiltInType =
            BuiltInType::from_str(&format!("{ty_ident}")).unwrap_or_default();
        return if built_in_ty.is_string() {
            (
                quote! {
                    let value = text_value;
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
                    let str_value = text_value;
                    let value : #ty = str_value.trim().parse()?;
                    #assignment
                },
                None,
            )
        };
    }
    (quote! {}, None)
}

pub fn create_assignments(f: &StructField) -> proc_macro2::TokenStream {
    let (assign_value, assign_empty_value) = create_assing_value(f);
    quote! {
        if is_empty {
            #assign_empty_value
        } else {
            let text_value = _raxb::de::read_text_content(reader, tag)?;
            #assign_value
        }
    }
}

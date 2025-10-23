use quote::quote;

use crate::{
    container::{BuiltInConstType, Container, EleType, Generic},
    utils::{get_built_in_const_type, get_built_in_type},
};

fn create_attribute_value_impl(ty: &syn::Type) -> proc_macro2::TokenStream {
    let built_in_ty = get_built_in_type(ty);
    if built_in_ty.is_string() {
        return quote! {
            value.as_ref()
        };
    }
    quote! {
        value.to_string().as_str()
    }
}

pub fn create_attribute_blocks(container: &Container) -> Vec<proc_macro2::TokenStream> {
    let mut blocks: Vec<proc_macro2::TokenStream> = Vec::new();
    for f in container
        .struct_fields
        .iter()
        .filter(|sf| sf.name.is_some() && matches!(sf.ty, EleType::Attr))
    {
        let name = f.name.as_ref().unwrap();
        let ident = f.original.ident.as_ref().unwrap();
        let v = name.value();
        let name =
            f.ns.as_ref()
                .map(|ns| ns.value())
                .map(|ns| {
                    format!(
                        "{}:{}",
                        std::str::from_utf8(&ns).unwrap(),
                        std::str::from_utf8(&v).unwrap()
                    )
                })
                .unwrap_or(String::from_utf8(v).unwrap());
        let ty = &f.original.ty;
        match f.generic {
            Generic::Vec(_) => {
                eprintln!("WARNING: Vec<T> cannot be used for attributes, use Option<T> instead");
            }
            Generic::Opt(ty) => {
                let attribute_value_impl = create_attribute_value_impl(ty);
                blocks.push(quote! {
                    if let Some(value) = self.#ident.as_ref() {
                        el_writer = el_writer.with_attribute((#name, {
                            #attribute_value_impl
                        }));
                    }
                })
            }
            Generic::None => {
                if f.value.is_some()
                    && matches!(get_built_in_const_type(ty), BuiltInConstType::ConstStr)
                {
                    let const_val = f.value.as_ref().unwrap();
                    let v = const_val.value();
                    let value = v.as_str();
                    blocks.push(quote! {
                        el_writer = el_writer.with_attribute((#name, #value));
                    })
                } else {
                    let attribute_value_impl = create_attribute_value_impl(ty);
                    blocks.push(quote! {
                        el_writer = el_writer.with_attribute((#name, {
                            let value = &self.#ident;
                            #attribute_value_impl
                        }));
                    })
                }
            }
        }
    }
    blocks
}

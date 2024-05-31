use quote::quote;

use crate::{
    container::{Container, EleType, Generic},
    utils::get_built_in_type,
};

fn write_text_value() -> proc_macro2::TokenStream {
    quote! {
        el_writer
            .write_text_content(_raxb::quick_xml::events::BytesText::from_escaped(_raxb::quick_xml::escape::escape(value)))?;
    }
}

fn write_text_value_ref() -> proc_macro2::TokenStream {
    quote! {
        el_writer
            .write_text_content(_raxb::quick_xml::events::BytesText::from_escaped(_raxb::quick_xml::escape::escape(&value)))?;
    }
}

fn write_any_value() -> proc_macro2::TokenStream {
    quote! {
        el_writer
            .write_text_content(_raxb::quick_xml::events::BytesText::from_escaped(_raxb::quick_xml::escape::escape(&value.to_string())))?;
    }
}

pub fn create_text_block(container: &Container) -> Option<proc_macro2::TokenStream> {
    if let Some(f) = container
        .struct_fields
        .iter()
        .find(|sf| matches!(sf.ty, EleType::Text))
    {
        let ident = f.original.ident.as_ref().unwrap();
        let ty = &f.original.ty;
        match f.generic {
            Generic::Vec(_) => {
                let write_value = write_text_value_ref();
                return Some(quote! {
                    let value = self.#ident.iter().map(|v| v.to_string()).join(",");
                    #write_value
                });
            }
            Generic::Opt(ty) => {
                let built_in_type = get_built_in_type(ty);
                if built_in_type.is_string() {
                    let write_value = write_text_value();
                    return Some(quote! {
                        if let Some(value) = self.#ident.as_ref() {
                            #write_value
                        }
                    });
                } else {
                    let write_value = write_any_value();
                    return Some(quote! {
                        if let Some(value) = self.#ident.as_ref() {
                            #write_value
                        }
                    });
                }
            }
            Generic::None => {
                let built_in_type = get_built_in_type(ty);
                if built_in_type.is_string() {
                    let write_value = write_text_value();
                    return Some(quote! {
                        let value = &self.#ident;
                        #write_value;
                    });
                } else {
                    let write_value = write_text_value_ref();
                    return Some(quote! {
                        let value = self.#ident.to_string();
                        #write_value
                    });
                }
            }
        }
    }
    None
}

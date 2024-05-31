use quote::quote;

use crate::{
    container::{Container, EleType, Generic},
    utils::get_built_in_type,
};

fn create_write_text_value(name: &str) -> proc_macro2::TokenStream {
    quote! {
        writer.create_element(#name)
            .write_text_content(_raxb::quick_xml::events::BytesText::from_escaped(_raxb::quick_xml::escape::escape(&value)))?;
    }
}

fn create_write_any_builtin_value(name: &str) -> proc_macro2::TokenStream {
    quote! {
        writer.create_element(#name)
            .write_text_content(_raxb::quick_xml::events::BytesText::from_escaped(_raxb::quick_xml::escape::escape(&value.to_string())))?;
    }
}

pub fn create_child_blocks(container: &Container) -> Vec<proc_macro2::TokenStream> {
    let mut blocks: Vec<proc_macro2::TokenStream> = Vec::new();
    for f in container.struct_fields.iter().filter(|sf| {
        sf.name.is_some() && matches!(sf.ty, EleType::Child | EleType::SelfClosedChild)
    }) {
        let ident = f.original.ident.as_ref().unwrap();

        let unqualified_name = f.name.as_ref().unwrap();
        let unqualified_name_buf = unqualified_name.value();
        let unqualified_name = std::str::from_utf8(&unqualified_name_buf).unwrap();

        let combined_name = if let Some(ns) = f.ns.as_ref() {
            let ns_buf = ns.value();
            let ns = std::str::from_utf8(&ns_buf).unwrap();
            std::borrow::Cow::Owned(format!("{ns}:{unqualified_name}"))
        } else {
            std::borrow::Cow::Borrowed(unqualified_name)
        };
        let name = combined_name.as_ref();
        let ty = &f.original.ty;
        let is_sfc = matches!(f.ty, EleType::SelfClosedChild);
        match f.generic {
            Generic::Vec(ty) => {
                let built_in_type = get_built_in_type(ty);
                if built_in_type.is_string() {
                    let write_value = create_write_text_value(name);
                    blocks.push(quote! {
                        for value in self.#ident.iter() {
                            #write_value
                        }
                    });
                } else if built_in_type.is_bool() {
                    if is_sfc {
                        blocks.push(quote! {
                            for value in self.#ident.iter() {
                                if value {
                                    writer.create_element(#name)
                                        .write_empty()?;
                                }
                            }
                        });
                    } else {
                        let write_value = create_write_any_builtin_value(name);
                        blocks.push(quote! {
                            for value in self.#ident.iter() {
                                #write_value
                            }
                        });
                    }
                } else if built_in_type.is_number() {
                    let write_value = create_write_any_builtin_value(name);
                    blocks.push(quote! {
                        for value in self.#ident.iter() {
                            #write_value
                        }
                    });
                } else if built_in_type.is_unknown() {
                    blocks.push(quote! {
                        for value in self.#ident.iter() {
                            value.xml_serialize(#name, writer)?;
                        }
                    });
                }
            }
            Generic::Opt(ty) => {
                let built_in_type = get_built_in_type(ty);
                if built_in_type.is_string() {
                    let write_value = create_write_text_value(name);
                    blocks.push(quote! {
                        if let Some(value) = self.#ident.as_ref() {
                            #write_value
                        }
                    })
                } else if built_in_type.is_bool() {
                    if is_sfc {
                        blocks.push(quote! {
                            if self.#ident.unwrap_or(false) {
                                writer.create_element(#name)
                                    .write_empty()?;
                            }
                        });
                    } else {
                        let write_value = create_write_any_builtin_value(name);
                        blocks.push(quote! {
                            if let Some(value) = self.#ident.as_ref() {
                                #write_value
                            }
                        });
                    }
                } else if built_in_type.is_number() {
                    let write_value = create_write_any_builtin_value(name);
                    blocks.push(quote! {
                        if let Some(value) = self.#ident.as_ref() {
                            #write_value
                        }
                    });
                } else if built_in_type.is_unknown() {
                    blocks.push(quote! {
                        if let Some(value) = self.#ident.as_ref() {
                            value.xml_serialize(#name, writer)?;
                        }
                    });
                }
            }
            Generic::None => {
                let built_in_type = get_built_in_type(ty);
                if built_in_type.is_string() {
                    blocks.push(quote! {
                        writer.create_element(#name)
                            .write_text_content(_raxb::quick_xml::events::BytesText::from_escaped(_raxb::quick_xml::escape::escape(&self.#ident)))?;
                    })
                } else if built_in_type.is_bool() {
                    if is_sfc {
                        blocks.push(quote! {
                            if self.#ident {
                                writer.create_element(#name)
                                    .write_empty()?;
                            }
                        });
                    } else {
                        blocks.push(quote! {
                            writer.create_element(#name)
                                .write_text_content(_raxb::quick_xml::events::BytesText::from_escaped(_raxb::quick_xml::escape::escape(&self.#ident.to_string())))?;
                        });
                    }
                } else if built_in_type.is_number() {
                    blocks.push(quote! {
                        writer.create_element(#name)
                            .write_text_content(_raxb::quick_xml::events::BytesText::from_escaped(_raxb::quick_xml::escape::escape(&self.#ident.to_string())))?;
                    });
                } else if built_in_type.is_unknown() {
                    blocks.push(quote! {
                        self.#ident.xml_serialize(#name, writer)?;
                    });
                }
            }
        }
    }
    blocks
}

use std::str::FromStr;

use crate::container::{BuiltInType, Container, EleType, Generic};
use quote::quote;

fn create_root_impl(container: &Container) -> proc_macro2::TokenStream {
    if let Some(root) = container.root.as_ref() {
        quote! {
            fn root() -> Option<_raxb::ty::XmlTag> {
                Some(#root)
            }
        }
    } else {
        quote! {}
    }
}

fn get_built_in_type(ty: &syn::Type) -> BuiltInType {
    if let syn::Type::Path(p) = ty {
        if let Some(ty_ident) = p.path.get_ident() {
            return BuiltInType::from_str(&format!("{ty_ident}")).unwrap_or_default();
        }
    }
    BuiltInType::Unknown
}

fn create_attribute_value_impl(ty: &syn::Type) -> proc_macro2::TokenStream {
    let built_in_ty = get_built_in_type(ty);
    if built_in_ty.is_string() {
        return quote! {
            _raxb::quick_xml::escape::escape(value).as_ref()
        };
    }
    quote! {
        _raxb::quick_xml::escape::escape(&value.to_string()).as_ref()
    }
}

fn create_attribute_blocks(container: &Container) -> Vec<proc_macro2::TokenStream> {
    let mut blocks: Vec<proc_macro2::TokenStream> = Vec::new();
    for f in container
        .struct_fields
        .iter()
        .filter(|sf| sf.name.is_some() && matches!(sf.ty, EleType::Attr))
    {
        let name = f.name.as_ref().unwrap();
        let ident = f.original.ident.as_ref().unwrap();
        let v = name.value();
        let name = std::str::from_utf8(&v).unwrap();
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
    blocks
}

fn create_text_block(container: &Container) -> Option<proc_macro2::TokenStream> {
    if let Some(f) = container
        .struct_fields
        .iter()
        .find(|sf| matches!(sf.ty, EleType::Text))
    {
        let ident = f.original.ident.as_ref().unwrap();
        let ty = &f.original.ty;
        match f.generic {
            Generic::Vec(_) => {
                return Some(quote! {
                    let value = self.#ident.iter().map(|v| v.to_string()).join(",");
                    el_writer
                        .write_text_content(_raxb::quick_xml::events::BytesText::from_escaped(_raxb::quick_xml::escape::escape(&value)))?;
                });
            }
            Generic::Opt(ty) => {
                let built_in_type = get_built_in_type(ty);
                if built_in_type.is_string() {
                    return Some(quote! {
                        if let Some(value) = self.#ident.as_ref() {
                            el_writer
                                .write_text_content(_raxb::quick_xml::events::BytesText::from_escaped(_raxb::quick_xml::escape::escape(&value)))?;
                        }
                    });
                } else {
                    return Some(quote! {
                        if let Some(value) = self.#ident.as_ref() {
                            el_writer
                                .write_text_content(_raxb::quick_xml::events::BytesText::from_escaped(_raxb::quick_xml::escape::escape(&value.to_string())))?;
                        }
                    });
                }
            }
            Generic::None => {
                let built_in_type = get_built_in_type(ty);
                if built_in_type.is_string() {
                    return Some(quote! {
                        el_writer
                            .write_text_content(_raxb::quick_xml::events::BytesText::from_escaped(_raxb::quick_xml::escape::escape(&self.#ident)))?;
                    });
                } else {
                    return Some(quote! {
                        el_writer
                            .write_text_content(_raxb::quick_xml::events::BytesText::from_escaped(_raxb::quick_xml::escape::escape(&self.#ident.to_string())))?;
                    });
                }
            }
        }
    }
    None
}

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

fn create_child_blocks(container: &Container) -> Vec<proc_macro2::TokenStream> {
    let mut blocks: Vec<proc_macro2::TokenStream> = Vec::new();
    for f in container.struct_fields.iter().filter(|sf| {
        sf.name.is_some() && matches!(sf.ty, EleType::Child | EleType::SelfClosedChild)
    }) {
        let name = f.name.as_ref().unwrap();
        let ident = f.original.ident.as_ref().unwrap();
        let v = name.value();
        let name = std::str::from_utf8(&v).unwrap();
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

fn create_root_element_impl(container: &Container) -> proc_macro2::TokenStream {
    let text_block = create_text_block(container);
    let child_blocks = if text_block.is_none() {
        create_child_blocks(container)
    } else {
        Vec::default()
    };
    let attribute_blocks = create_attribute_blocks(container);
    let has_attributes = !attribute_blocks.is_empty();
    let mut has_child_blocks = !child_blocks.is_empty();

    let children = if let Some(text_block) = text_block {
        has_child_blocks = true;
        text_block
    } else if has_child_blocks {
        let child_blocks = child_blocks.into_iter();
        quote! {
            el_writer
                .write_inner_content::<_, _raxb::ser::XmlSerializeError>(|writer| {
                    #(#child_blocks)*
                    Ok(())
                })?;
        }
    } else {
        quote! {
            el_writer.write_empty()?;
        }
    };
    if has_attributes || has_child_blocks {
        let attribute_blocks = attribute_blocks.into_iter();
        quote! {
            let mut el_writer = writer.create_element(tag);
            #(#attribute_blocks)*
            #children
        }
    } else {
        quote! {
            writer.create_element(tag).write_empty()?;
        }
    }
}

pub fn impl_block(container: Container) -> proc_macro2::TokenStream {
    let root_impl = create_root_impl(&container);
    let create_root_element = create_root_element_impl(&container);
    let ident = &container.original.ident;
    let (impl_generics, type_generics, where_clause) = container.original.generics.split_for_impl();
    quote! {
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute, clippy::manual_flatten, clippy::single_match)]
            extern crate raxb as _raxb;

            #[automatically_derived]
            impl #impl_generics _raxb::ser::XmlSerialize for #ident #type_generics #where_clause {
                #root_impl
                fn xml_serialize<W: std::io::Write>(&self, tag: &str, writer: &mut _raxb::quick_xml::Writer<W>) -> _raxb::ser::XmlSerializeResult<()> {
                    #create_root_element
                    Ok(())
                }
            }
        };
    }
}

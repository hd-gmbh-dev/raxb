use quote::quote;

use crate::{container::Container, utils::get_built_in_type};

fn create_root_impl(container: &Container) -> proc_macro2::TokenStream {
    if let Some(root) = container.root.as_ref() {
        if let Some((prefix, _)) = container.tns.as_ref() {
            let prefix_buf = prefix.value();
            let prefix = std::str::from_utf8(&prefix_buf).unwrap();
            let root_buf = root.value();
            let root = std::str::from_utf8(&root_buf).unwrap();
            let tag: syn::LitByteStr = syn::parse_str(&format!("b\"{prefix}:{root}\"")).unwrap();
            quote! {
                fn root() -> Option<_raxb::ty::XmlTag> {
                    Some(#tag)
                }
            }
        } else {
            quote! {
                fn root() -> Option<_raxb::ty::XmlTag> {
                    Some(#root)
                }
            }
        }
    } else {
        quote! {}
    }
}

pub fn impl_block(container: Container) -> proc_macro2::TokenStream {
    let root_impl = create_root_impl(&container);
    let ident = &container.original.ident;
    let serialize_branches = container.enum_variants.iter().filter_map(|variant| {
        let variant_ident = variant.ident;
        if let Some((name, ty)) = variant.name.as_ref().zip(variant.ty.as_ref()) {
            let v = name.value();
            let name = if let Some(ns) = variant.ns.as_ref() {
                let ns_buf = ns.value();
                format!(
                    "{}:{}",
                    String::from_utf8_lossy(&ns_buf),
                    String::from_utf8_lossy(&v)
                )
            } else {
                String::from_utf8(v).unwrap()
            };
            let built_in_type = get_built_in_type(ty);
            if built_in_type.is_bool() {
                return Some(quote! {
                    Self::#variant_ident(v) => {
                        if v {
                            writer.create_element(#name).write_empty()?;
                        }
                        Ok::<(), _raxb::ser::XmlSerializeError>(())
                    }
                });
            } else if built_in_type.is_number() {
                return Some(quote! {
                    Self::#variant_ident(v) => {
                        writer.create_element(#name).write_text_content(
                            &v.to_string()
                        )?;
                        Ok::<(), _raxb::ser::XmlSerializeError>(())
                    }
                });
            } else if built_in_type.is_string() {
                return Some(quote! {
                    Self::#variant_ident(v) => {
                        writer.create_element(#name).write_text_content(
                            _raxb::quick_xml::events::BytesText::from_escaped(
                                _raxb::quick_xml::escape::escape(&v),
                            ),
                        )?;
                        Ok::<(), _raxb::ser::XmlSerializeError>(())
                    }
                });
            } else {
                return Some(quote! {
                    Self::#variant_ident(v) => {
                        v.xml_serialize(#name, writer)?;
                        Ok::<(), _raxb::ser::XmlSerializeError>(())
                    }
                });
            }
        } else if let Some(name) = variant.name.as_ref() {
            let v = name.value();
            let name = if let Some(ns) = variant.ns.as_ref() {
                let ns_buf = ns.value();
                format!(
                    "{}:{}",
                    String::from_utf8_lossy(&ns_buf),
                    String::from_utf8_lossy(&v)
                )
            } else {
                String::from_utf8(v).unwrap()
            };
            return Some(quote! {
                Self::#variant_ident => {
                    writer.create_element(#name).write_empty()?;
                    Ok::<(), _raxb::ser::XmlSerializeError>(())
                }
            });
        }
        None
    });
    let (impl_generics, type_generics, where_clause) = container.original.generics.split_for_impl();
    let serialize_branches_1 = serialize_branches.clone();
    let serialize_branches_2 = serialize_branches;
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
                    if tag.is_empty() {
                        match self {
                            #(#serialize_branches_1,)*
                        }?;
                    } else {
                        writer.create_element(tag).write_inner_content(|writer| {
                            match self {
                                #(#serialize_branches_2,)*
                            }
                        })?;
                    }
                    Ok::<(), _raxb::ser::XmlSerializeError>(())
                }

                fn is_enum() -> bool {
                    true
                }
            }
        };
    }
}

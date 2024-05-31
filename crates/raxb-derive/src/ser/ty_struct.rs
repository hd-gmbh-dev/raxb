use quote::quote;

use crate::container::Container;

use super::{child::create_child_blocks, text::create_text_block};

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

fn create_root_element_impl(container: &Container) -> proc_macro2::TokenStream {
    let text_block = create_text_block(container);
    let child_blocks = if text_block.is_none() {
        create_child_blocks(container)
    } else {
        Vec::default()
    };
    let attribute_blocks = super::attrs::create_attribute_blocks(container);
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
    let create_el = if let Some((prefix, ns)) = container.tns.as_ref() {
        let prefix_buf = prefix.value();
        let prefix = std::str::from_utf8(&prefix_buf).unwrap();
        let ns_buf = ns.value();
        let ns = std::str::from_utf8(&ns_buf).unwrap();
        let key = if !prefix.is_empty() {
            format!("xmlns:{prefix}")
        } else {
            "xmlns".to_string()
        };
        quote! { writer.create_element(tag).with_attribute((#key, #ns)) }
    } else {
        quote! { writer.create_element(tag) }
    };
    if has_attributes || has_child_blocks {
        let attribute_blocks = attribute_blocks.into_iter();
        quote! {
            let mut el_writer = #create_el;
            #(#attribute_blocks)*
            #children
        }
    } else {
        quote! {
            #create_el.write_empty()?;
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

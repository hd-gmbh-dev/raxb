use quote::quote;
use syn::LitByteStr;

use crate::container::{Container, EleType, FieldsSummary, StructField};

fn create_return_value(fields: &[StructField]) -> proc_macro2::TokenStream {
    let branch = fields.iter().map(|f| {
        let ident = f.original.ident.as_ref().unwrap();
        let field_name: LitByteStr = syn::parse_str(&format!("b\"{ident}\"")).unwrap();
        if f.is_required() {
            if matches!(f.ty, EleType::Attr) {
                quote! {
                    #ident: #ident.ok_or(_raxb::de::XmlDeserializeError::MissingAttribute(_raxb::de::S(#field_name)))?,
                }
            } else {
                quote! {
                    #ident: #ident.ok_or(_raxb::de::XmlDeserializeError::MissingElement(_raxb::de::S(#field_name)))?,
                }
            }
        } else {
            quote! {
                #ident,
            }
        }
    });
    quote! {#(#branch)*}
}

pub fn impl_block(container: Container) -> proc_macro2::TokenStream {
    let root_impl = create_root_impl(&container);
    let field_assignments = if let Some(f) = container
        .struct_fields
        .iter()
        .find(|sf| matches!(sf.ty, EleType::Text))
    {
        super::text::create_assignments(f)
    } else {
        super::child::create_assignments(&container)
    };
    let return_value = create_return_value(&container.struct_fields);
    let summary = FieldsSummary::from_fields(container.struct_fields);
    let fields_init = create_fields_init(&summary);
    let attr_assignments = super::attrs::create_assignments(&summary);
    let ident = &container.original.ident;
    let (impl_generics, type_generics, where_clause) = container.original.generics.split_for_impl();
    quote! {
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute, clippy::manual_flatten, clippy::single_match)]
            extern crate raxb as _raxb;

            #[automatically_derived]
            impl #impl_generics _raxb::de::XmlDeserialize for #ident #type_generics #where_clause {
                fn xml_deserialize<R: std::io::BufRead>(
                    reader: &mut _raxb::quick_xml::NsReader<R>,
                    target_ns: _raxb::de::XmlTag,
                    tag: _raxb::de::XmlTargetNs,
                    attributes: _raxb::quick_xml::events::attributes::Attributes,
                    is_empty: bool,
                ) -> _raxb::de::XmlDeserializeResult<Self> {

                    #fields_init
                    #attr_assignments
                    #field_assignments

                    Ok(Self {
                        #return_value
                    })
                }
                #root_impl
            }
        };
    }
}

fn create_root_impl(container: &Container) -> proc_macro2::TokenStream {
    if let Some(root) = container.root.as_ref() {
        quote! {
            fn root() -> Option<_raxb::de::XmlTag> {
                Some(#root)
            }
        }
    } else {
        quote! {}
    }
}

fn create_fields_init(fields: &FieldsSummary) -> proc_macro2::TokenStream {
    let attrs_init = super::attrs::init(fields);
    let childs_init = super::child::init(fields);
    let text_init = super::text::init(fields);

    quote! {
        #attrs_init
        #childs_init
        #text_init
    }
}

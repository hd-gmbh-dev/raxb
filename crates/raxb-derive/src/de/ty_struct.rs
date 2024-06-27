use quote::quote;
use syn::LitByteStr;

use crate::{
    container::{Container, EleType, FieldsSummary, StructField},
    utils::{create_root_impl, create_tns_impl},
};

fn create_return_value(fields: &[StructField]) -> proc_macro2::TokenStream {
    let branch = fields.iter().map(|f| {
        let ident = f.original.ident.as_ref().unwrap();
        let field_name: LitByteStr = syn::parse_str(&format!("b\"{ident}\"")).unwrap();
        if f.default {
            quote! {
                #ident: #ident.unwrap_or_default(),
            }
        }
        else if f.is_required() {
            if matches!(f.ty, EleType::Attr) {
                quote! {
                    #ident: #ident.ok_or(_raxb::de::XmlDeserializeError::MissingAttribute(_raxb::ty::S(#field_name)))?,
                }
            } else {
                quote! {
                    #ident: #ident.ok_or(_raxb::de::XmlDeserializeError::MissingElement(_raxb::ty::S(#field_name)))?,
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
    let tns_impl = create_tns_impl(&container);
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
            use _raxb::{
                de::{XmlDeserialize, XmlDeserializeError, XmlDeserializeResult},
                quick_xml::{
                    events::{attributes::Attributes, Event},
                    name::ResolveResult,
                    NsReader,
                },
                ty::{XmlTag, XmlTargetNs, S},
            };
            #[automatically_derived]
            impl #impl_generics XmlDeserialize for #ident #type_generics #where_clause {
                fn xml_deserialize<R: std::io::BufRead>(
                    reader: &mut NsReader<R>,
                    target_ns: XmlTag,
                    tag: XmlTargetNs,
                    attributes: Attributes,
                    is_empty: bool,
                ) -> XmlDeserializeResult<Self> {
                    let target_ns = Self::target_ns().unwrap_or(target_ns);

                    #fields_init
                    #attr_assignments
                    #field_assignments

                    Ok(Self {
                        #return_value
                    })
                }
                #root_impl
                #tns_impl
            }
        };
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

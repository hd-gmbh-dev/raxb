use quote::quote;

use crate::container::Container;

pub fn impl_block(_container: Container) -> proc_macro2::TokenStream {
    quote! {}
}

use syn::DeriveInput;

mod reflect;
mod type_uuid;
use quote::quote;

#[proc_macro_derive(Component, attributes(uuid, reflect))]
pub fn component_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    let reflect = reflect::reflect_derive(&ast);
    let type_uuid = type_uuid::type_uuid_derive(&ast);
    quote! {
        #reflect
        #type_uuid
    }
    .into()
}

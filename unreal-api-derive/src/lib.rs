use syn::DeriveInput;

mod component;
mod event;
mod reflect;
mod type_uuid;
use quote::quote;

#[proc_macro_derive(Component, attributes(uuid, reflect))]
pub fn component_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    let reflect = reflect::reflect_derive(&ast);
    let type_uuid = type_uuid::type_uuid_derive(&ast);
    let component = component::component_derive(&ast);
    quote! {
        #reflect
        #type_uuid
        #component
    }
    .into()
}

#[proc_macro_derive(Event, attributes(uuid, reflect))]
pub fn event_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    let reflect = reflect::reflect_derive(&ast);
    let type_uuid = type_uuid::type_uuid_derive(&ast);
    let event = event::event_derive(&ast);
    quote! {
        #reflect
        #type_uuid
        #event
    }
    .into()
}

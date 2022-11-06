extern crate proc_macro;

use proc_macro2::Span;
use quote::quote;
use syn::*;
pub(crate) fn event_derive(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    if let Data::Struct(_data) = &ast.data {
        let struct_ident = &ast.ident;
        let send_event_ident = Ident::new(&format!("{}SendEvent", ast.ident), Span::call_site());
        let register_event = quote! {
            impl SendEntityEvent for #send_event_ident {
                fn send_entity_event(&self, world: &mut World, entity: Entity, json: &str) {
                    let event: #struct_ident = serde_json::de::from_str(json).unwrap();
                    world.send_event(EntityEvent { entity, event });
                }
            }
            impl RegisterEvent for #struct_ident {
                fn register_event(registry: &mut ReflectionRegistry) {
                    registry.send_entity_event.insert(Self::TYPE_UUID, Box::new(#send_event_ident));
                }
            }
        };
        quote! {

            const _:() = {
                use unreal_api::core::SendEntityEvent;
                use unreal_api::*;
                use unreal_api::module::*;
                use unreal_api::ecs::component::{Component, TableStorage};

                struct #send_event_ident;

                #register_event

                // TODO: This should not be here. Events are not components, but reflect currently requires it.
                impl Component for #struct_ident {
                    type Storage = TableStorage;
                }
            };
        }
    } else {
        panic!()
    }
}

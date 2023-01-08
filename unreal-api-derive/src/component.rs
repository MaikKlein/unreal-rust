extern crate proc_macro;

use darling::FromDeriveInput;
use proc_macro2::Span;
use quote::quote;
use syn::*;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(reflect))]
pub struct ReflectEditor {
    #[darling(default)]
    editor: bool,
}

pub fn component_derive(ast: &DeriveInput) -> proc_macro2::TokenStream {
    let is_editor_component =
        ReflectEditor::from_derive_input(ast).map_or(false, |reflect| reflect.editor);
    if let Data::Struct(_data) = &ast.data {
        let struct_ident = &ast.ident;
        let add_serialized_ident = Ident::new(
            &format!("{}AddSerializedComponent", ast.ident),
            Span::call_site(),
        );
        let register_add_serialized_component = if is_editor_component {
            quote! {
                impl AddSerializedComponent for #add_serialized_ident {
                    unsafe fn add_serialized_component(
                        &self,
                        json: &str,
                        commands: &mut bevy_ecs::system::EntityCommands<'_, '_, '_>,
                    ) {
                        let component = unreal_api::serde_json::de::from_str::<#struct_ident>(json).expect(json);
                        commands.insert(component);
                    }
                }

                impl RegisterSerializedComponent for #struct_ident {
                    fn register_serialized_component(registry: &mut ReflectionRegistry) {
                        registry.insert_serialized_component.insert(
                            <#struct_ident as TypeUuid>::TYPE_UUID,
                            Box::new(#add_serialized_ident),
                        );
                    }
                }
            }
        } else {
            quote! {}
        };

        quote! {

            const _:() = {
                use unreal_api::*;
                use unreal_api::ecs::component::{Component, TableStorage};
                use unreal_api::module::*;
                use unreal_api::editor_component::*;

                struct #add_serialized_ident;

                impl Component for #struct_ident {
                    type Storage = TableStorage;
                }

                #register_add_serialized_component
            };
        }
    } else {
        panic!()
    }
}

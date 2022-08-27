extern crate proc_macro;

use darling::{FromDeriveInput, FromField};
use proc_macro2::Span;
use quote::quote;
use syn::*;

#[derive(Debug, FromField)]
#[darling(attributes(reflect))]
pub struct ReflectField {
    ident: Option<Ident>,
    ty: Type,
    #[darling(default)]
    skip: bool,
}
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(reflect))]
pub struct ReflectEditor {
    #[darling(default)]
    editor: bool,
}

pub fn reflect_derive(ast: &DeriveInput) -> proc_macro2::TokenStream {
    let is_editor_component =
        ReflectEditor::from_derive_input(ast).map_or(false, |reflect| reflect.editor);

    if let Data::Struct(data) = &ast.data {
        let literal_name = LitStr::new(&ast.ident.to_string(), Span::call_site());
        let reflect_struct_ident = Ident::new(&format!("{}Reflect", ast.ident), Span::call_site());
        let insert_struct_ident =
            Ident::new(&format!("{}InsertComponent", ast.ident), Span::call_site());
        let struct_ident = &ast.ident;

        let fields: Vec<ReflectField> = data
            .fields
            .iter()
            .map(|field| ReflectField::from_field(field).unwrap())
            .collect();

        let reflect_fields: Vec<&ReflectField> =
            fields.iter().filter(|field| !field.skip).collect();

        let number_of_fields = reflect_fields.len() as u32;

        let field_indices: Vec<u32> = (0..number_of_fields).collect();
        let field_types: Vec<&'_ Type> = reflect_fields.iter().map(|field| &field.ty).collect();
        let field_names: Vec<LitStr> = reflect_fields
            .iter()
            .map(|field| {
                let ident = field.ident.as_ref().unwrap();
                LitStr::new(&ident.to_string(), Span::call_site())
            })
            .collect();

        let field_idents: Vec<&'_ Ident> = reflect_fields
            .iter()
            .map(|field| field.ident.as_ref().unwrap())
            .collect();

        let self_ty: Type = syn::parse_str(&ast.ident.to_string()).unwrap();

        let field_methods = if number_of_fields > 0 {
            quote! {
                fn get_field_type(&self, idx: u32) -> Option<unreal_api::registry::ReflectType> {
                    match idx {
                        #(
                            #field_indices => Some(<#field_types as unreal_api::registry::ReflectStatic>::TYPE),
                            )*
                            _ => None
                    }
                }
                fn get_field_name(&self, idx: u32) -> Option<&'static str> {
                    match idx {
                        #(
                            #field_indices => Some(#field_names),
                        )*
                        _ => None
                    }
                }
                fn has_component(&self, world: &unreal_api::World, entity: unreal_api::Entity) -> bool {
                    world
                        .get_entity(entity)
                        .and_then(|entity_ref| entity_ref.get::<#self_ty>()).is_some()
                }
                fn get_field_value(&self, world: &unreal_api::World, entity: unreal_api::Entity, idx: u32) -> Option<unreal_api::registry::ReflectValue> {
                    world
                        .get_entity(entity)
                        .and_then(|entity_ref| entity_ref.get::<#self_ty>())
                        .and_then(|component| {
                            let ty = match idx {
                                #(
                                    #field_indices => component.#field_idents.get_value(),
                                )*
                                _ => return None,
                            };
                            Some(ty)
                        })
                }
            }
        } else {
            quote!()
        };

        let insert_component = if is_editor_component {
            quote! {
                impl unreal_api::editor_component::InsertEditorComponent for #insert_struct_ident {
                    unsafe fn insert_component(
                        &self,
                        actor: *const unreal_api::ffi::AActorOpaque,
                        uuid: unreal_api::uuid::Uuid,
                        commands: &mut unreal_api::ecs::system::EntityCommands<'_, '_, '_>,
                    ) {
                        use unreal_api::editor_component::GetEditorComponentValue;
                        let component = #struct_ident {
                            #(
                                #field_idents: #field_types::get(actor, uuid, #field_names).expect(#field_names),
                            )*
                        };
                        commands.insert(component);
                    }
                }
            }
        } else {
            quote! {}
        };

        let register_editor_component = if is_editor_component {
            quote! {
                registry.insert_editor_component.insert(
                    <#struct_ident as unreal_api::TypeUuid>::TYPE_UUID,
                    Box::new(#insert_struct_ident),
                );

            }
        } else {
            quote!()
        };

        quote! {
            pub struct #reflect_struct_ident;

            impl unreal_api::registry::ReflectDyn for #reflect_struct_ident {
                fn name(&self) -> &'static str {
                    #literal_name
                }

                fn number_of_fields(&self) -> u32 {
                    #number_of_fields
                }

                #field_methods

                fn get_value(&self) -> unreal_api::registry::ReflectValue {
                    unreal_api::registry::ReflectValue::Composite
                }

            }
            impl unreal_api::registry::ReflectStatic for #reflect_struct_ident {
                const TYPE: unreal_api::registry::ReflectType = unreal_api::registry::ReflectType::Composite;
            }
            impl unreal_api::ecs::component::Component for #struct_ident {
                type Storage = unreal_api::ecs::component::TableStorage;
            }
            pub struct #insert_struct_ident;
            #insert_component

            impl unreal_api::module::InsertReflectionStruct for #struct_ident {
                fn insert(registry: &mut unreal_api::module::ReflectionRegistry) {
                    registry.reflect.insert(
                        <#struct_ident as unreal_api::TypeUuid>::TYPE_UUID,
                        Box::new(#reflect_struct_ident),
                    );
                    #register_editor_component

                }
            }
        }
    } else {
        panic!("Only structs are currently supported in `unreal_api_derive`")
    }
}

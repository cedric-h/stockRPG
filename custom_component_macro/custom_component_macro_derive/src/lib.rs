extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(AssemblageComponent)]
pub fn assemblage_component_macro_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_assemblage_component_macro(&ast)
}

fn impl_assemblage_component_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        #[typetag::serde]
        impl AssemblageComponent for #name {
            fn add_to_lazy_builder(&self, builder: &specs::world::LazyBuilder) {
                let entity = builder.entity;
                let component = self.clone();
                builder
                    .lazy.exec(move |world| {
                        if world.write_storage().insert(entity, component).is_err() {
                            warn!(
                                "Lazy insert of component failed because {:?} was dead.",
                                entity
                            );
                        }
                    });
            }
            fn boxed_clone(&self) -> Box<dyn AssemblageComponent> { Box::new(self.clone()) }
        }
    };
    gen.into()
}

#[proc_macro_derive(DevUiComponent)]
pub fn dev_ui_component_macro_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_dev_ui_component_macro(&ast)
}

fn impl_dev_ui_component_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl DevUiComponent for #name {
            fn ui_for_entity(&self, ui: &imgui::Ui, world: &specs::World, ent: &specs::Entity) {
                let mut mes = world.write_storage::<Self>();
                let requested = mes.get_mut(*ent).unwrap();
                requested.dev_ui_render(&ui, &world);
            }
        }
    };
    gen.into()
}

#[proc_macro_derive(CopyToOtherEntity)]
pub fn copy_to_other_entity_macro_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_copy_to_other_entity_macro(&ast)
}

fn impl_copy_to_other_entity_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl CopyToOtherEntity for #name {
            fn copy_self_to(&self, world: &specs::World, ent: &specs::Entity) {
                let mut mes = world.write_storage::<Self>();
                mes.insert(*ent, self.clone()).unwrap();
            }
        }
    };
    gen.into()
}

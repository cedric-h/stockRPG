use mopa::*;

#[typetag::serde]
pub trait AssemblageComponent: DevUiComponent + CopyToOtherEntity + mopa::Any + std::fmt::Debug + std::marker::Send + std::marker::Sync {
    //fn world_insert(&self, entity: Entity, world: &World) -> InsertResult<()>;
    //fn world_remove(&self, entity: Entity, world: &World);
    //fn lazy_remove(&self, entity: Entity, lazy_update: &specs::Read<specs::LazyUpdate>);
    //fn lazy_insert(&self, entity: Entity, lazy_update: &Read<LazyUpdate>);
    //fn add_to_builder(&self, builder: &specs::EntityBuilder) -> specs::storage::InsertResult<()>;

    fn add_to_lazy_builder(&self, builder: &specs::world::LazyBuilder);
    fn boxed_clone(&self) -> Box<dyn AssemblageComponent>;
}
mopafy!(AssemblageComponent);

pub trait CopyToOtherEntity {
    fn copy_self_to(&self, world: &specs::World, ent: &specs::Entity);
}

pub trait DevUiComponent: DevUiRender {
    fn ui_for_entity(&self, ui: &imgui::Ui, world: &specs::World, ent: &specs::Entity);
}

pub trait DevUiRender {
    fn dev_ui_render(&mut self, ui: &imgui::Ui, world: &specs::World);
}

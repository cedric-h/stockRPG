use comp_prelude::*;

#[derive(
    Component,
    DevUiComponent,
    CopyToOtherEntity,
    AssemblageComponent,
    Serialize,
    Deserialize,
    Clone,
    Debug,
)]
#[storage(HashMapStorage)]
pub struct Explodeable {
    pub delete_entity: bool,
    pub delete_component: bool,
    pub chunks_count: i32,
    pub force: ApplyForce,
}
impl Default for Explodeable {
    fn default() -> Self {
        Self {
            delete_entity: true,
            delete_component: false,
            chunks_count: 7,
            force: ApplyForce {
                vec: glm::vec3(12.5, 0.0, 0.0),
                ..ApplyForce::default()
            },
        }
    }
}
impl DevUiRender for Explodeable {
    fn dev_ui_render(&mut self, ui: &imgui::Ui, world: &specs::World) {
        use imgui::*;

        ui.text(im_str!("Explodeable"));

        ui.checkbox(
            im_str!("< delete entity after explode"),
            &mut self.delete_entity,
        );
        ui.checkbox(
            im_str!("< delete component after explode"),
            &mut self.delete_component,
        );

        ui.input_int(im_str!("chunks count"), &mut self.chunks_count)
            .step(1)
            .build();

        ui.text(im_str!("Giblet Force: "));
        ui.text(im_str!(
            "Note, only the first value in the force \nconfiguration is used."
        ));
        self.force.dev_ui_render(ui, world);
    }
}


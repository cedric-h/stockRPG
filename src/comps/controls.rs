use comp_prelude::*;

#[derive(
    Default,
    Component,
    DevUiComponent,
    CopyToOtherEntity,
    AssemblageComponent,
    Serialize,
    Deserialize,
    Debug,
    Clone,
)]
#[storage(HashMapStorage)]
pub struct MovementControls {
    pub speed: f32,
}
impl DevUiRender for MovementControls {
    fn dev_ui_render(&mut self, ui: &imgui::Ui, _world: &specs::World) {
        use imgui::*;

        ui.text(im_str!("MovementControls"));

        ui.slider_float(im_str!("speed"), &mut self.speed, 0.0, 20.0)
            .build();
    }
}



use super::prelude::*;

//health component
#[derive(
    Default,
    Component,
    DevUiComponent,
    CopyToOtherEntity,
    AssemblageComponent,
    Serialize,
    Deserialize,
    Clone,
    Debug,
)]
#[storage(VecStorage)]
pub struct Health {
    pub value: f32,
    pub max: f32,
}
impl DevUiRender for Health {
    fn dev_ui_render(&mut self, ui: &imgui::Ui, _world: &specs::World) {
        use imgui::*;

        ui.text(im_str!("Health"));
        ui.input_float(im_str!("current value"), &mut self.value)
            .build();
        ui.input_float(im_str!("max"), &mut self.max).build();
    }
}

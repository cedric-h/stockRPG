use comp_prelude::*;

//this component gives the compendium and save/load system a reference point for entity composition

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
#[storage(DenseVecStorage)]
pub struct Assemblaged {
    pub built_from: String,
}
impl DevUiRender for Assemblaged {
    fn dev_ui_render(&mut self, ui: &imgui::Ui, _world: &specs::World) {
        ui.text(imgui::im_str!("Assembled From: "));
    }
}

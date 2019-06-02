use super::prelude::*;
use imgui::*;

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
pub struct Member {
    group_id: u32,
    member_id: u32,
}
impl DevUiRender for Member {
    fn dev_ui_render(&mut self, ui: &imgui::Ui, _world: &specs::World) {
        ui.text(im_str!("Member"));
    }
}

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
#[storage(VecStorage)]
pub struct Tile {
    left: u32,
    right: u32,
    above: u32,
    below: u32
}
impl DevUiRender for Tile {
    fn dev_ui_render(&mut self, ui: &imgui::Ui, _world: &specs::World) {
        ui.text(im_str!("Tile"));
    }
}

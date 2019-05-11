use comp_prelude::*;

// rendering related components!

#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct Appearance {
    pub uv_rect: [f32; 4],
}

#[derive(
    Default, Component, CopyToOtherEntity, AssemblageComponent, Serialize, Deserialize, Debug, Clone,
)]
#[storage(VecStorage)]
pub struct AppearanceBuilder {
    pub image_name: String,
    pub uv_override: [f32; 4],
    pub built: bool,
}
//AnimationBuilder is weird, because they aren't actually used for anything, and are immediately
//turned into Animations when detected
impl DevUiComponent for AppearanceBuilder {
    fn ui_for_entity(&self, ui: &imgui::Ui, world: &specs::World, ent: &specs::Entity) {
        let mut appearances = world.write_storage::<Appearance>();
        let app = appearances.get_mut(*ent).unwrap();
        use imgui::*;

        ui.text(im_str!("Appearance"));

        for (index, coord) in app.uv_rect.iter().enumerate() {
            ui.label_text(im_str!("uv coordinate #{}", index), im_str!("{}", coord));
        }
    }
}
impl DevUiRender for AppearanceBuilder {
    fn dev_ui_render(&mut self, ui: &imgui::Ui, world: &specs::World) {
        use imgui::*;
        let image_bundle = world.read_resource::<ImageBundle>();

        ui.text(im_str!("Appearance"));

        let image_names = image_bundle
            .map
            .keys()
            .map(|x| ImString::new(x.clone()))
            .collect::<Vec<_>>();

        let image_im_str_names = image_names.iter().map(ImStr::new).collect::<Vec<_>>();

        let mut name_index =
            if let Some(index) = image_bundle.map.keys().position(|x| *x == self.image_name) {
                index
            } else {
                ui.text(im_str!(
                    "error, couldn't find the name of the image being used."
                ));
                0
            } as i32;

        if ui.combo(
            im_str!("Image Names"),
            &mut name_index,
            &image_im_str_names,
            12,
        ) {
            self.image_name = image_im_str_names[name_index as usize].to_str().to_owned();
        }

        let size: &mut [f32; 2] = &mut [self.uv_override[2], self.uv_override[3]];
        ui.input_float(im_str!("uv coord 1"), &mut self.uv_override[0])
            .step(32.0)
            .build();
        ui.input_float(im_str!("uv coord 2"), &mut self.uv_override[1])
            .step(32.0)
            .build();
        ui.drag_float2(im_str!("uv size"), size).build();
        self.uv_override = [self.uv_override[0], self.uv_override[1], size[0], size[1]];
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
pub struct Animation {
    pub frame_count: i32,
    pub fps: f32,
}
impl DevUiRender for Animation {
    fn dev_ui_render(&mut self, ui: &imgui::Ui, _world: &specs::World) {
        use imgui::*;

        ui.text(im_str!("Animation"));

        ui.drag_float(im_str!("fps"), &mut self.fps)
            .power(0.01)
            .speed(0.01)
            .build();
        ui.input_int(im_str!("frame count"), &mut self.frame_count)
            .step(1)
            .build();
    }
}

#[derive(
    Default,
    Component,
    CopyToOtherEntity,
    DevUiComponent,
    AssemblageComponent,
    Serialize,
    Deserialize,
    Clone,
    Debug,
)]
#[storage(VecStorage)]
pub struct CameraFocus {
    pub background_color: [f32; 4],
    pub zoom: f32,
    pub interpolation_speed: f32,
}
impl DevUiRender for CameraFocus {
    fn dev_ui_render(&mut self, ui: &imgui::Ui, world: &specs::World) {
        use imgui::*;

        ui.text(im_str!("CameraFocus"));

        ui.input_float(im_str!("follow speed"), &mut self.interpolation_speed)
            .step(0.01)
            .build();

        if ui
            .input_float(im_str!("zoom"), &mut self.zoom)
            .step(0.01)
            .build()
        {
            world
                .write_resource::<LocalState>()
                .camera
                .set_zoom(self.zoom);
        }

        ui.color_edit(im_str!("Background fill color"), &mut self.background_color)
            .format(ColorFormat::Float)
            .build();
    }
}


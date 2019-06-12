//use super::draw_data::*;
//use super::helper;
use crate::prelude::*;
use image::RgbaImage;

pub struct GameRenderer {
}

impl GameRenderer {
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::D32Float;

    pub fn init(display: &Display) -> Self {
    }

    fn update_clear_color(&mut self, world: &specs::World) {
        use specs::Join;
        let cam_fs = world.read_storage::<CameraFocus>();

        let fill = &cam_fs
            .join()
            .next()
            .map(|cf| cf.background_color)
            .unwrap_or([0.1, 0.2, 0.3, 1.0]);

        self.clear_color = Color {
            r: fill[0],
            g: fill[1],
            b: fill[2],
            a: fill[3],
        };
    }

    pub fn resize(&mut self, sc_desc: &SwapChainDescriptor, device: &mut Device) {
    }
    pub fn render(
        &mut self,
        world: &specs::World,
        device: &mut Device,
        encoder: &mut Encoder,
        view: &TextureView,
    ) -> Result<(), String> {
    }
}

fn get_view_projection(world: &specs::World) -> glm::TMat4<f32> {
    let ls = world.read_resource::<LocalState>();
    ls.perspective_projection * ls.camera.view_matrix
}

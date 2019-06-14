use super::draw_data::*;
//use super::helper;
use crate::prelude::*;
use glium::{
    index::PrimitiveType,
    program,
    texture::{RawImage2d, SrgbTexture2d},
    uniform,
    uniforms::*,
    Depth, DepthTest, Display, DrawParameters, Frame, IndexBuffer, VertexBuffer,
};
use image::RgbaImage;

const DEFAULT_BLUE: [f32; 4] = [0.1, 0.2, 0.3, 1.0];

pub struct GameRenderer {
    spritesheet_program: glium::Program,
    //outline_program: glium::Program,
    opengl_texture: SrgbTexture2d,
    index_buffer: IndexBuffer<u16>,
    clear_color: [f32; 4],
}

impl GameRenderer {
    pub fn init(image: RgbaImage, display: &Display) -> Result<Self, String> {
        let image_dimensions = image.dimensions();
        let image = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
        let opengl_texture = SrgbTexture2d::new(display, image).unwrap();

        let index_buffer =
            IndexBuffer::new(display, PrimitiveType::TrianglesList, &[0, 1, 2, 2, 1, 3])
                .map_err(|_| "Couldn't initialize quad index buffer")?;

        Ok(Self {
            spritesheet_program: load_shaders(
                display,
                include_str!("./spritesheet.vert"),
                include_str!("./spritesheet.frag"),
            ).unwrap(),
            opengl_texture,
            index_buffer,
            clear_color: DEFAULT_BLUE,
        })
    }

    #[inline]
    fn update_clear_color(&mut self, world: &specs::World) {
        use specs::Join;
        let cam_fs = world.read_storage::<CameraFocus>();

        self.clear_color = cam_fs
            .join()
            .next()
            .map(|cf| cf.background_color)
            .unwrap_or(DEFAULT_BLUE);
    }

    pub fn render(
        &mut self,
        target: &mut Frame,
        display: &Display,
        world: &specs::World,
    ) -> Result<(), String> {
        self.update_clear_color(world);
        use glium::Surface;

        let draw_params = DrawParameters {
            depth: Depth {
                test: DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            ..Default::default()
        };

        //fill in background
        let c = self.clear_color;
        target.clear_color_and_depth((c[0], c[1], c[2], c[3]), 1.0);

        let uniforms = uniform! {
            matrix: {
                let m: [[f32; 4]; 4] = *get_view_projection(world).as_ref();
                m
            },
            tex: Sampler::new(&self.opengl_texture)
                .wrap_function(SamplerWrapFunction::Repeat)
                .magnify_filter(MagnifySamplerFilter::Nearest)
                .minify_filter(MinifySamplerFilter::Nearest),
        };

        //render all of the normal sprites
        for verts in SpritesheetVertex::get_from_ecs(world) {
            let vertex_buffer = VertexBuffer::new(display, &verts)
                .map_err(|_| "Couldn't create the quad vertex buffer")?;
            target
                .draw(
                    &vertex_buffer,
                    &self.index_buffer,
                    &self.spritesheet_program,
                    &uniforms,
                    &draw_params,
                )
                .map_err(|_| "Vertex render pass failed!")?;
        }

        Ok(())
    }
}

#[inline]
fn get_view_projection(world: &specs::World) -> glm::TMat4<f32> {
    let ls = world.read_resource::<LocalState>();
    ls.perspective_projection * ls.camera.view_matrix
}

#[inline]
fn load_shaders(
    display: &Display,
    frag_str: &str,
    vert_str: &str,
) -> Result<glium::Program, String> {
    program!(
        display,
        140 => {
            vertex: frag_str,
            fragment: vert_str,
        },
    )
    .map_err(|e| format!("shader error: {:?}", e))
}

//mod draw_data;
//mod game_renderer;
//mod helper;

pub struct SpritesheetDimensions {
    pub x: f32,
    pub y: f32,
}

use crate::prelude::*;
//pub use draw_data::*;
//use game_renderer::GameRenderer;

use glium::Display;
use glutin::{ContextCurrentState, WindowedContext};
use imgui_glium_renderer as im_glium;

pub static IMG_BYTES: &[u8] = include_bytes!("../img/spritesheet.png");

pub struct GliumState {
    //game_renderer: GameRenderer,
    imgui_renderer: im_glium::Renderer,
    display: Display,
}

impl GliumState {
    pub fn new<T: ContextCurrentState>(
        window: WindowedContext<T>,
        imgui: &mut imgui::ImGui,
    ) -> (Self, SpritesheetDimensions) {
        let display = Display::unchecked(window).unwrap();
        let window = display.gl_window();

        //imgui renderer
        let imgui_renderer =
            im_glium::Renderer::init(imgui, &display).expect("failed to initialize imgui renderer");

        //game renderer
        // Create the texture
        let texels = image::load_from_memory(IMG_BYTES)
            .expect("Binary corrupted!")
            .to_rgba();
        let spritesheet_dimensions = SpritesheetDimensions {
            x: texels.dimensions().0 as f32,
            y: texels.dimensions().1 as f32,
        };

        //finally instantiate the tuple we'll return
        (
            Self {
                display,
                imgui_renderer,
            },
            spritesheet_dimensions,
        )
    }

    #[inline]
    fn resize_if_should(&mut self, world: &specs::World) {
        let ls = world.read_resource::<LocalState>();
        if let Some((x, y)) = ls.last_input.new_frame_size {
            self.resize(x, y);
        }
    }

    #[inline]
    pub fn resize(&mut self, x: f64, y: f64) {
        /*
        self.swap_chain_descriptor.width = x.round() as u32;
        self.swap_chain_descriptor.height = y.round() as u32;
        self.swap_chain = self
            .device
            .create_swap_chain(&self.surface, &self.swap_chain_descriptor);
        self.game_renderer
            .resize(&self.swap_chain_descriptor, &mut self.device);*/
    }

    pub fn render(&mut self, world: &specs::World, ui: imgui::Ui) {
        use glium::Surface;
        let mut target = self.display.draw();
        target.clear_color(0.1, 0.2, 0.3, 1.0);
        self.imgui_renderer
            .render(&mut target, ui)
            .expect("imgui rendering failed");
        target.finish().unwrap();
    }
}

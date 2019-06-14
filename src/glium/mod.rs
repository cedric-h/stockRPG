mod draw_data;
mod game_renderer;
//mod helper;

pub struct SpritesheetDimensions {
    pub x: f32,
    pub y: f32,
}

use crate::prelude::*;
//pub use draw_data::*;
use game_renderer::GameRenderer;

use glium::Display;
use imgui_glium_renderer as im_glium;

pub static IMG_BYTES: &[u8] = include_bytes!("../img/spritesheet.png");

pub struct GliumState {
    game_renderer: GameRenderer,
    imgui_renderer: im_glium::Renderer,
    pub display: Display,
}

impl GliumState {
    pub fn new(
        window: glutin::GlWindow,
        imgui: &mut imgui::ImGui,
    ) -> (Self, SpritesheetDimensions) {
        let display = Display::from_gl_window(window).unwrap();
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

        let game_renderer =
            GameRenderer::init(texels, &display).expect("failed to initialize game renderer!");

        //finally instantiate the tuple we'll return
        drop(window);
        (
            Self {
                display,
                imgui_renderer,
                game_renderer,
            },
            spritesheet_dimensions,
        )
    }

    pub fn render(&mut self, world: &specs::World, ui: imgui::Ui) {
        let mut target = self.display.draw();

        self.game_renderer
            .render(&mut target, &self.display, world)
            .expect("game rendering failed");
        self.imgui_renderer
            .render(&mut target, ui)
            .expect("imgui rendering failed");

        target.finish().unwrap();
    }
}

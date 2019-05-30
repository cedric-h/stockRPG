mod draw_data;
mod game_renderer;
mod helper;

pub use draw_data::*;
use crate::prelude::*;
use game_renderer::GameRenderer;

pub static IMG_BYTES: &[u8] = include_bytes!("../img/spritesheet.png");

pub struct WgpuState {
    #[allow(dead_code)]
    instance: wgpu::Instance,
    #[allow(dead_code)]
    adapter: wgpu::Adapter,
    game_renderer: GameRenderer,
    imgui_renderer: imgui_wgpu::Renderer,
    device: wgpu::Device,
    surface: wgpu::Surface,
    swap_chain: wgpu::SwapChain,
    swap_chain_descriptor: wgpu::SwapChainDescriptor,
}

impl WgpuState {
    pub fn new(window: &winit::Window, imgui: &mut imgui::ImGui) -> (Self, SpritesheetDimensions) {
        let instance = wgpu::Instance::new();
        let adapter = instance.get_adapter(&wgpu::AdapterDescriptor {
            power_preference: wgpu::PowerPreference::LowPower,
        });
        let mut device = adapter.request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false,
            },
            limits: wgpu::Limits::default(),
        });
        let size = window
            .get_inner_size()
            .unwrap()
            .to_physical(window.get_hidpi_factor());
        let surface = instance.create_surface(window);
        let swap_chain_descriptor = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: size.width.round() as u32,
            height: size.height.round() as u32,
        };
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_descriptor);

        //game renderer
        // Create the texture
        let texels = image::load_from_memory(IMG_BYTES)
            .expect("Binary corrupted!")
            .to_rgba();
        let spritesheet_dimensions = SpritesheetDimensions {
            x: texels.dimensions().0 as f32,
            y: texels.dimensions().1 as f32,
        };
        let game_renderer = GameRenderer::init(texels, &swap_chain_descriptor, &mut device);

        //imgui renderer
        let format = wgpu::TextureFormat::Bgra8Unorm;
        let imgui_renderer = imgui_wgpu::Renderer::new(imgui, &mut device, format, None)
            .expect("Couldn't make imgui renderer");

        (
            Self {
                game_renderer,
                imgui_renderer,
                instance,
                adapter,
                device,
                surface,
                swap_chain,
                swap_chain_descriptor,
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
        self.swap_chain_descriptor.width = x.round() as u32;
        self.swap_chain_descriptor.height = y.round() as u32;
        self.swap_chain = self
            .device
            .create_swap_chain(&self.surface, &self.swap_chain_descriptor);
        self.game_renderer
            .resize(&self.swap_chain_descriptor, &mut self.device);
    }

    pub fn render(&mut self, world: &specs::World, ui: imgui::Ui) {
        self.resize_if_should(world);

        // make encoder & frame
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        let frame = self.swap_chain.get_next_texture();

        // call game renderer
        self.game_renderer
            .render(world, &mut self.device, &mut encoder, &frame.view)
            .expect("game rendering failed");

        // and now let's imgui_render
        self.imgui_renderer
            .render(ui, &mut self.device, &mut encoder, &frame.view)
            .expect("imgui rendering failed");

        // submit encoder
        self.device.get_queue().submit(&[encoder.finish()]);
    }
}

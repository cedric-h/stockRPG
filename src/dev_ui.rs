//imgui
use imgui::{FontGlyphRange, ImFontConfig, ImGui, ImVec4, Ui};
use imgui_gfx_renderer::{Renderer, Shaders};
use imgui_winit_support;
//time
use std::time::Instant;
//gfx
use gfx::format::{Unorm, D24_S8, R8_G8_B8_A8};
use gfx::handle::{DepthStencilView, RenderTargetView};
use gfx::{self, Device};
use gfx_device_gl::{CommandBuffer, Resources};
use gfx_window_glutin;
//glutin
use glutin;

pub struct DevUiState {
    events_loop: glutin::EventsLoop,
    window: glutin::GlWindow,
    encoder: gfx::Encoder<Resources, CommandBuffer>,
    device: gfx_device_gl::Device,
    factory: gfx_device_gl::Factory,
    main_color: RenderTargetView<Resources, (R8_G8_B8_A8, Unorm)>,
    main_depth: DepthStencilView<Resources, (D24_S8, Unorm)>,
    pub clear_color: [f32; 4],
    imgui: ImGui,
    hidpi_factor: f64,
    renderer: Renderer<Resources>,
    last_frame: Instant,
}

impl DevUiState {
    pub fn new() -> Self {
        type ColorFormat = gfx::format::Rgba8;
        type DepthFormat = gfx::format::DepthStencil;

        let events_loop = glutin::EventsLoop::new();
        let context = glutin::ContextBuilder::new(); //.with_vsync(true);
        let window = glutin::WindowBuilder::new()
            .with_title("Developer UI")
            .with_dimensions(glutin::dpi::LogicalSize::new(525.0, 625.0));
        let (window, device, mut factory, main_color, main_depth) =
            gfx_window_glutin::init::<ColorFormat, DepthFormat>(window, context, &events_loop)
                .expect("Failed to initalize graphics");
        let encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
        let shaders = {
            let version = device.get_info().shading_language;
            if version.is_embedded {
                if version.major >= 3 {
                    Shaders::GlSlEs300
                } else {
                    Shaders::GlSlEs100
                }
            } else if version.major >= 4 {
                Shaders::GlSl400
            } else if version.major >= 3 {
                if version.minor >= 2 {
                    Shaders::GlSl150
                } else {
                    Shaders::GlSl130
                }
            } else {
                Shaders::GlSl110
            }
        };

        let mut imgui = ImGui::init();
        {
            // Fix incorrect colors with sRGB framebuffer
            fn imgui_gamma_to_linear(col: ImVec4) -> ImVec4 {
                let x = col.x.powf(2.2);
                let y = col.y.powf(2.2);
                let z = col.z.powf(2.2);
                let w = 1.0 - (1.0 - col.w).powf(2.2);
                ImVec4::new(x, y, z, w)
            }

            let style = imgui.style_mut();
            for col in 0..style.colors.len() {
                style.colors[col] = imgui_gamma_to_linear(style.colors[col]);
            }
        }
        imgui.set_ini_filename(None);

        // In the examples we only use integer DPI factors, because the UI can get very blurry
        // otherwise. This might or might not be what you want in a real application.
        let hidpi_factor = window.get_hidpi_factor().round();

        let font_size = (8.0 * hidpi_factor) as f32;

        imgui.fonts().add_font_with_config(
            include_bytes!("./font/SDS_8x8.ttf"),
            ImFontConfig::new()
                .oversample_h(1)
                .pixel_snap_h(true)
                .size_pixels(font_size),
            &FontGlyphRange::default(),
        );

        imgui.set_font_global_scale((1.0 / hidpi_factor) as f32);

        let renderer = Renderer::init(&mut imgui, &mut factory, shaders, main_color.clone())
            .expect("Failed to initialize renderer");

        imgui_winit_support::configure_keys(&mut imgui);

        Self {
            events_loop,
            window,
            encoder,
            device,
            factory,
            main_color,
            main_depth,
            clear_color: [0.1, 0.2, 0.3, 1.0],
            imgui,
            hidpi_factor,
            renderer,
            last_frame: Instant::now(),
        }
    }

    pub fn update<F: FnMut(&Ui)>(&mut self, mut run_ui: F) {
        let imgui = &mut self.imgui;
        let window = &mut self.window;
        let hidpi_factor = &mut self.hidpi_factor;
        let main_color = &mut self.main_color;
        let main_depth = &mut self.main_depth;
        let renderer = &mut self.renderer;

        self.events_loop.poll_events(|event| {
            use glutin::{Event, WindowEvent::Resized};

            imgui_winit_support::handle_event(
                imgui,
                &event,
                window.get_hidpi_factor(),
                *hidpi_factor,
            );

            if let Event::WindowEvent { event, .. } = event {
                match event {
                    Resized(_) => {
                        gfx_window_glutin::update_views(window, main_color, main_depth);
                        renderer.update_render_target(main_color.clone());
                    }
                    _ => (),
                }
            }
        });

        let now = Instant::now();
        let delta = now - self.last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        self.last_frame = now;

        imgui_winit_support::update_mouse_cursor(imgui, window);

        let frame_size = imgui_winit_support::get_frame_size(window, self.hidpi_factor).unwrap();

        let ui = imgui.frame(frame_size, delta_s);
        run_ui(&ui);

        self.encoder.clear(&self.main_color, self.clear_color);
        self.renderer
            .render(ui, &mut self.factory, &mut self.encoder)
            .expect("Rendering failed");
        self.encoder.flush(&mut self.device);
        self.window.swap_buffers().unwrap();
        self.device.cleanup();
    }
}

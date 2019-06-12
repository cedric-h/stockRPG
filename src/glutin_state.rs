// use crate::prelude::*;
use crate::prelude::*;
use glutin::{
    dpi::LogicalSize, ContextCurrentState, CreationError, Event, EventsLoop, VirtualKeyCode,
    Window, WindowBuilder, WindowedContext,
};
use std::collections::HashSet;

#[derive(Debug)]
pub struct GlutinState {
    pub events_loop: EventsLoop,
    pub keys_held: HashSet<VirtualKeyCode>,
}

impl GlutinState {
    /// Constructs a new `EventsLoop` and `Window` pair.
    ///
    /// The specified title and size are used, other elements are default.
    /// ## Failure
    /// It's possible for the window creation to fail. This is unlikely.
    pub fn new<T: Into<String>, CCS: ContextCurrentState>(
        title: T,
        size: LogicalSize,
    ) -> Result<(Self, WindowedContext<CCS>), CreationError> {
        let events_loop = EventsLoop::new();
        let wb = WindowBuilder::new().with_title(title).with_dimensions(size);
        glutin::ContextBuilder::new()
            .with_vsync(true)
            .build_windowed(wb, &events_loop)
            .map(|window| {
                (
                    Self {
                        events_loop,
                        keys_held: HashSet::new(),
                    },
                    window,
                )
            })
    }

    pub fn input(&mut self, world: &specs::World, dev_ui: &mut DevUiState) {
        // gotta store that input somewhere, so the rest of the program can access it! :D
        let mut local_state = world.write_resource::<LocalState>();
        let mut input_frame = UserInput::default();

        // manually split borrow
        let events_loop = &mut self.events_loop;
        let window = &self.window;
        let keys_held = &mut self.keys_held;

        // sometimes I wonder why imgui doesn't just record this and be done with it.
        let dpi_factor = self.window.get_hidpi_factor().round();

        //this is mostly just resize if needed.
        dev_ui.other_input_processing(window);

        events_loop.poll_events(|event| {
            dev_ui.process_event(&event, dpi_factor);

            if let Event::WindowEvent {
                event: win_event, ..
            } = event
            {
                // this probably won't crash, don't worry
                let io = unsafe { &*imgui::sys::igGetIO() };

                match win_event {
                    glutin::WindowEvent::KeyboardInput { .. } if io.want_capture_keyboard => {}

                    glutin::WindowEvent::MouseInput { .. }
                    | glutin::WindowEvent::MouseWheel { .. }
                        if io.want_capture_mouse => {}

                    _ => input_frame.process_event(&win_event, keys_held),
                };
            }
        });

        input_frame.keys_held = keys_held.clone();
        local_state.update_from_input(input_frame);
    }
}

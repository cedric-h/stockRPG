// use crate::prelude::*;
use crate::prelude::*;
use std::collections::HashSet;
use winit::{
    dpi::LogicalSize, CreationError, Event, EventsLoop, VirtualKeyCode, Window, WindowBuilder,
};

#[derive(Debug)]
pub struct WinitState {
    pub events_loop: EventsLoop,
    pub window: Window,
    pub keys_held: HashSet<VirtualKeyCode>,
}

impl WinitState {
    /// Constructs a new `EventsLoop` and `Window` pair.
    ///
    /// The specified title and size are used, other elements are default.
    /// ## Failure
    /// It's possible for the window creation to fail. This is unlikely.
    pub fn new<T: Into<String>>(title: T, size: LogicalSize) -> Result<Self, CreationError> {
        let events_loop = EventsLoop::new();
        let output = WindowBuilder::new()
            .with_title(title)
            .with_dimensions(size)
            .build(&events_loop);
        output.map(|window| Self {
            events_loop,
            window,
            keys_held: HashSet::new(),
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
                    winit::WindowEvent::KeyboardInput { .. } if io.want_capture_keyboard => {}

                    winit::WindowEvent::MouseInput { .. }
                    | winit::WindowEvent::MouseWheel { .. }
                        if io.want_capture_mouse => {}

                    _ => input_frame.process_event(&win_event, keys_held),
                };
            }
        });

        input_frame.keys_held = keys_held.clone();
        local_state.update_from_input(input_frame);
    }
}

impl Default for WinitState {
    /// Makes an 800x600 window with the `WINDOW_NAME` value as the title.
    /// ## Panics
    /// If a `CreationError` occurs.
    fn default() -> Self {
        Self::new(
            "stockRPG",
            LogicalSize {
                width: 1366.0,
                height: 768.0,
            },
        )
        .expect("Could not create a window!")
    }
}

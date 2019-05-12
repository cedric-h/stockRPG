//use crate::prelude::*;
use crate::prelude::*;
use std::collections::HashSet;
use winit::{
    Event, dpi::LogicalSize, CreationError,
    EventsLoop, VirtualKeyCode, Window,
    WindowBuilder,
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
        let mut local_state = world.write_resource::<LocalState>();
        let mut input_frame = UserInput::default();
        let events_loop = &mut self.events_loop;
        let keys_held = &mut self.keys_held;
        let game_window_id = &self.window.id();

        events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent {
                    window_id: id,
                    ..
                } => {
                    if id == *game_window_id {
                        input_frame.process_event(&event, keys_held);
                    } else if id == dev_ui.window.id() {
                        dev_ui.process_event(&event);
                    }
                },
                _ => {
                    dev_ui.process_event(&event);
                    input_frame.process_event(&event, keys_held);
                }
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
                width: 800.0,
                height: 600.0,
            },
        )
        .expect("Could not create a window!")
    }
}

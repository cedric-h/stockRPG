//use crate::prelude::*;
use winit::{
    dpi::LogicalSize, CreationError, EventsLoop,
    VirtualKeyCode, Window, WindowBuilder,
};
use std::collections::HashSet;

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

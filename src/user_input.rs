use crate::prelude::*;
use glutin::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};
use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct UserInput {
    pub end_requested: bool,
    pub new_frame_size: Option<(f64, f64)>,
    pub keys_held: HashSet<VirtualKeyCode>,
    pub mouse_pos: Option<(f32, f32)>,
    pub mouse_state: Option<bool>,
    pub seconds: f32,
    pub focus: Option<bool>,
}

impl UserInput {
    pub fn process_event(&mut self, event: &WindowEvent, keys_held: &mut HashSet<VirtualKeyCode>) {
        // now we actually poll those events
        match event {
            // Close when asked
            WindowEvent::CloseRequested => self.end_requested = true,

            // Track all keys, all the time. Note that because of key rollover details
            // it's possible to get key released events for keys we don't think are
            // pressed. This is a hardware limit, not something you can evade.
            // :: any key event code used to be here ::
            // We want to respond to some of the keys specially when they're also
            // window events too (meaning that the window was focused when the event
            // happened).
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(code),
                        ..
                    },
                ..
            } => {
                // apparently on macs we can only get key events when the window is focused,
                // but that's fine since that's all we want anyway.
                match state {
                    ElementState::Pressed => keys_held.insert(*code),
                    ElementState::Released => keys_held.remove(&code),
                };
            }

            WindowEvent::CursorMoved {
                position: glutin::dpi::LogicalPosition { x, y },
                ..
            } => {
                self.mouse_pos = Some((*x as f32, *y as f32));
            }

            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                self.mouse_state = Some(true);
            }

            WindowEvent::Focused(focus_state) => {
                self.focus = Some(*focus_state);
            }

            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                self.mouse_state = Some(false);
            }

            // Update our size info if the window changes size.
            WindowEvent::Resized(logical) => {
                self.new_frame_size = Some((logical.width, logical.height));
            }

            _ => (),
        };
    }
}

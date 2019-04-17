use crate::prelude::*;
use winit::{
    VirtualKeyCode, KeyboardInput, WindowEvent, Event, MouseButton, ElementState,
};
use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct UserInput {
    pub end_requested: bool,
    pub new_frame_size: Option<(f64, f64)>,
    pub swap_projection: bool,
    pub keys_held: HashSet<VirtualKeyCode>,
    pub mouse_pos: Option<(f32, f32)>,
    pub mouse_state: Option<bool>,
    pub seconds: f32,
}

impl UserInput {
    pub fn poll_events_loop(winit_state: &mut WinitState) -> Self {
        let mut output = UserInput::default();
        // We have to manually split the borrow here. rustc, why you so dumb sometimes?
        let events_loop = &mut winit_state.events_loop;
        let keys_held = &mut winit_state.keys_held;
        // now we actually poll those events
        events_loop.poll_events(|event| match event {
            // Close when asked
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => output.end_requested = true,

            // Track all keys, all the time. Note that because of key rollover details
            // it's possible to get key released events for keys we don't think are
            // pressed. This is a hardware limit, not something you can evade.
            // :: any key event code used to be here ::
            // We want to respond to some of the keys specially when they're also
            // window events too (meaning that the window was focused when the event
            // happened).
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state,
                                virtual_keycode: Some(code),
                                ..
                            },
                            ..
                    },
                    ..
            } => {
                //apparently on macs we can only get key events when the window is focused,
                //but that's fine since that's all we want anyway.
                match state {
                    ElementState::Pressed => keys_held.insert(code),
                    ElementState::Released => keys_held.remove(&code),
                };
                if state == ElementState::Pressed {
                    match code {
                        VirtualKeyCode::Tab => output.swap_projection = !output.swap_projection,
                        _ => (),
                    }
                }
            }

            Event::WindowEvent {
                event: WindowEvent::CursorMoved {
                    position: winit::dpi::LogicalPosition { x, y },
                    ..
                },
                ..
            } => {
                output.mouse_pos = Some((x as f32, y as f32));
            }

            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        state: ElementState::Pressed,
                        button: MouseButton::Left,
                        ..
                    },
                    ..
            } => {
                output.mouse_state = Some(true);
            }

            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        state: ElementState::Released,
                        button: MouseButton::Left,
                        ..
                    },
                    ..
            } => {
                output.mouse_state = Some(false);
            }

            // Update our size info if the window changes size.
            Event::WindowEvent {
                event: WindowEvent::Resized(logical),
                ..
            } => {
                output.new_frame_size = Some((logical.width, logical.height));
            }

            _ => (),
        });
        output.keys_held = keys_held.clone();
        output
    }
}


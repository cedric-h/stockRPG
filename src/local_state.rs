use crate::prelude::*;
use std::collections::HashSet;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct LocalState {
    pub frame_width: f64,
    pub frame_height: f64,
    pub camera: Camera,
    pub perspective_projection: glm::TMat4<f32>,
    pub last_update: std::time::Instant,
    pub elapsed_time: f32,
    pub last_frame_duration: f32,
    pub mouse_pos: (f32, f32),
    pub mouse_down: bool,
    pub quit: bool,
    pub last_input: UserInput,
    pub tapped_keys: HashSet<glutin::VirtualKeyCode>,
    pub focused: bool,
}

impl LocalState {
    pub fn from_glutin_window(window: &glutin::Window) -> Self {
        let (frame_width, frame_height) = window
            .get_inner_size()
            .map(|logical| logical.into())
            .unwrap_or((0.0, 0.0));
        Self {
            frame_width,
            frame_height,
            elapsed_time: 0.0,
            last_frame_duration: 0.0,
            quit: false,
            last_update: std::time::Instant::now(),
            camera: Camera::at_position(glm::vec3(0.0, 0.0, 0.0)),
            perspective_projection: LocalState::get_perspective(frame_width, frame_height),
            mouse_pos: (0.0, 0.0),
            mouse_down: false,
            last_input: UserInput::default(),
            tapped_keys: std::collections::HashSet::new(),
            focused: true,
        }
    }

    pub fn find_camera_focus_and_zoom(&mut self, world: &specs::World) {
        use specs::Join;

        let camera_focuses = world.read_storage::<CameraFocus>();
        if let Some(foc) = camera_focuses.join().next() {
            self.camera.set_zoom(foc.zoom);
        } else {
            info!("No camera focus found in the ECS world after loading the save file.");
        }
    }

    pub fn update_from_input(&mut self, input: UserInput) {
        // these events only matter if the window is focused when they're emitted.
        // also, these are done before self.focused is updated, because sometimes
        // mouse events return a false 0, 0, position when the click causes focus.
        if self.focused {
            if let Some(mouse_pos) = input.mouse_pos {
                if mouse_pos.0 != 0.0 && mouse_pos.1 != 0.0 {
                    self.mouse_pos = mouse_pos;
                }
            }
            if let Some(mouse_down) = input.mouse_state {
                self.mouse_down = mouse_down;
            }
        }

        if input.end_requested {
            self.quit = true;
        }
        if let Some(focus_state) = input.focus {
            self.focused = focus_state;
        }
        if let Some(frame_size) = input.new_frame_size {
            self.frame_width = frame_size.0;
            self.frame_height = frame_size.1;
            self.update_perspective();
        }
        assert!(self.frame_width != 0.0 && self.frame_height != 0.0);

        let now = Instant::now();
        let duration = now.duration_since(self.last_update);
        let duration = duration.as_secs() as f32 + duration.subsec_nanos() as f32 * 1e-9;
        self.last_update = now;
        self.elapsed_time += duration;
        self.last_frame_duration = duration;

        // figure out which keys were just tapped based on the keys that were pressed last frame and
        // the keys that are pressed now. if a key is pressed now, but it wasn't pressed last frame,
        // well it'd have to be fresh pressed, yeah? and that's the best coffee.
        self.tapped_keys.clear();
        for key in &input.keys_held {
            if !self.last_input.keys_held.contains(key) {
                self.tapped_keys.insert(*key);
            }
        }

        self.last_input = input;
        // self.camera.update_position(&input.keys_held, 5.0 * duration);
    }
    pub fn update_perspective(&mut self) {
        self.perspective_projection =
            LocalState::get_perspective(self.frame_width, self.frame_height);
    }
    pub fn get_perspective(frame_width: f64, frame_height: f64) -> glm::TMat4<f32> {
        let mut temp = glm::perspective_lh_zo(
            (frame_width / frame_height) as f32,
            f32::to_radians(10.0),
            0.1,
            1000.0,
        );
        temp[(1, 1)] *= -1.0;
        temp
    }
}

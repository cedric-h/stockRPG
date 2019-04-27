use crate::prelude::*;

/// Top down style camera.
///
/// The perspective version of this could potentially be problematic, we'll see.
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    /// Camera position, free free to update directly at any time.
    pub position: glm::TVec3<f32>,
    pub offset: glm::TVec3<f32>,
    quat: glm::Qua<f32>,
    pub view_matrix: glm::TMat4<f32>,
}
impl Camera {
    /// Updates the position of the camera with WASDQE controls.
    ///
    /// All motion is relative to the current orientation.
    pub fn lerp_towards(&mut self, focus: glm::TVec3<f32>, coefficient: f32) {
        self.position = glm::lerp_vec(
            &self.position,
            &focus,
            &(glm::vec3(1.0, 1.0, 1.0) * coefficient),
        );
        self.update_view_matrix();
    }

    /// Generates the current view matrix for this camera.
    fn get_view_matrix(&self) -> glm::TMat4<f32> {
        let rotation = glm::quat_to_mat4(&self.quat);
        let translation = glm::translation(&(self.position + self.offset));
        glm::inverse(&(translation * rotation))
    }

    fn update_view_matrix(&mut self) {
        self.view_matrix = self.get_view_matrix();
    }

    pub fn get_quat(&self) -> glm::Qua<f32> {
        self.quat
    }

    /// Makes a new camera at the position specified and an identity orientation.
    pub fn at_position(position: glm::TVec3<f32>) -> Self {
        let mut cam = Self {
            position,
            offset: glm::vec3(0.0, 55.0, 22.5) * 0.5,
            quat: glm::quat_normalize(&glm::quat(
                -0.8260998,
                -0.004217835,
                -0.006799023,
                -0.563467,
            )),
            view_matrix: glm::identity(),
        };
        cam.update_view_matrix();
        cam
    }
}

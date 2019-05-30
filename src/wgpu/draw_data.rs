use crate::prelude::*;
use nalgebra::{Point3};

#[derive(Clone, Copy)]
pub struct Vertex {
    _pos: [f32; 4],
    _tex_coord: [f32; 2],
}

pub struct SpritesheetDimensions {
    pub x: f32,
    pub y: f32,
}

pub fn vertex(pos: Point3<f32>, tc: &[f32; 2]) -> Vertex {
    Vertex {
        _pos: [pos.x, pos.y, pos.z, 1.0],
        _tex_coord: [tc[0], tc[1]],
    }
}

pub struct DrawData {
    /// Drawables; things to draw; entities with the right components
    pub vertexes: Vec<Vec<Vertex>>,

    /// Camera!
    pub view_projection: glm::TMat4<f32>,
}

use cgmath::{Matrix4, SquareMatrix};

use crate::models::{Camera, CameraProjection};

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    // We can't use cgmath with bytemuck directly so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera, projection: &CameraProjection) {
        self.view_position = camera.position.to_homogeneous().into();
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into();
    }
}

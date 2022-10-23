use cgmath::{Matrix4, perspective, Rad};

use crate::models::camera::OPENGL_TO_WGPU_MATRIX;

pub struct CameraProjection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl CameraProjection {
    pub fn new<F>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self
        where F: Into<Rad<f32>>
    {
        #[allow(clippy::cast_precision_loss)]
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

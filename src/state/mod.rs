use wgpu::{BindGroup, Device, Queue, RenderPipeline, Surface, SurfaceConfiguration};
use winit::dpi::PhysicalSize;

pub use render::render;

use crate::models::{Camera, CameraConfiguration, CameraController, Geometry};

mod render;
mod state_static;
mod state_impl;

pub struct State {
    camera: Camera,
    camera_configuration: CameraConfiguration,
    camera_controller: CameraController,
    device: Device,
    diffuse_bind_group: BindGroup,
    geometry: Geometry,
    queue: Queue,
    render_pipeline: RenderPipeline,
    size: PhysicalSize<u32>,
    surface: Surface,
    surface_configuration: SurfaceConfiguration,
}

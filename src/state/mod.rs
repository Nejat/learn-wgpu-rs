use wgpu::{BindGroup, Buffer, Device, Queue, RenderPipeline, Surface, SurfaceConfiguration};
use winit::dpi::PhysicalSize;

pub use render::render;

use crate::models::{Camera, CameraConfiguration, CameraController, Geometry, Instance};

mod render;
mod state_static;
mod state_impl;
mod initialize;

pub struct State {
    camera: Camera,
    camera_configuration: CameraConfiguration,
    camera_controller: CameraController,
    device: Device,
    diffuse_bind_group: BindGroup,
    geometry: Geometry,
    instances: Vec<Instance>,
    instance_buffer: Buffer,
    queue: Queue,
    render_pipeline: RenderPipeline,
    size: PhysicalSize<u32>,
    surface: Surface,
    surface_configuration: SurfaceConfiguration,
}

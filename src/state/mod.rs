use wgpu::{Buffer, Device, Queue, RenderPipeline, Surface, SurfaceConfiguration};
use winit::dpi::PhysicalSize;

use crate::models::{Camera, CameraConfiguration, CameraController, Instance, Light, Model, Texture};

mod state_static;
mod state_impl;
mod initialize;

pub struct State {
    camera: Camera,
    camera_configuration: CameraConfiguration,
    camera_controller: CameraController,
    depth_texture: Texture,
    device: Device,
    instances: Vec<Instance>,
    instance_buffer: Buffer,
    light: Light,
    light_pipeline: RenderPipeline,
    obj_model: Model,
    queue: Queue,
    render_pipeline: RenderPipeline,
    size: PhysicalSize<u32>,
    surface: Surface,
    surface_configuration: SurfaceConfiguration,
}

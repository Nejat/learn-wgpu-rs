use wgpu::{Buffer, Device, Queue, RenderPipeline, Surface, SurfaceConfiguration};

use crate::models::{
    Camera, CameraConfiguration, CameraController, CameraProjection,
    Instance, Light, Model, Texture,
};

mod state_static;
mod state_impl;
mod initialize;

pub struct State {
    camera: Camera,
    camera_configuration: CameraConfiguration,
    camera_controller: CameraController,
    camera_projection: CameraProjection,
    depth_texture: Texture,
    device: Device,
    instances: Vec<Instance>,
    instance_buffer: Buffer,
    light: Light,
    light_pipeline: RenderPipeline,
    mouse_pressed: bool,
    obj_model: Model,
    queue: Queue,
    render_pipeline: RenderPipeline,
    surface: Surface,
    surface_configuration: SurfaceConfiguration,
}

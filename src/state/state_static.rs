use cgmath::Vector3;
use wgpu::{Backends, Instance};
use winit::window::Window;

use crate::models::{Camera, CameraConfiguration, CameraController, Texture};
use crate::resources::load_model;
use crate::State;
use crate::state::initialize::{
    diffuse_bind_group_layout, get_instances, render_pipeline, request_adapter,
    request_device, surface_configuration,
};

impl State {
    // Creating some of the wgpu types requires async code
    #[cfg_attr(target_arch = "wasm32", allow(clippy::future_not_send))] // todo: winit window is not send
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = request_adapter(&instance, &surface).await;
        let (device, queue) = request_device(&adapter).await;
        let surface_configuration = surface_configuration(&adapter, &device, &surface, size);
        let diffuse_bind_group_layout = diffuse_bind_group_layout(&device, "diffuse-texture");
        let camera_controller = CameraController::new(0.2);
        let (instances, instance_buffer) = get_instances(&device);
        #[allow(clippy::cast_precision_loss)]
        let aspect = surface_configuration.width as f32 / surface_configuration.height as f32;

        let camera = Camera {
            aspect,
            // position the camera one unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, 5.0, -10.0).into(),
            fov_y: 45.0,
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: Vector3::unit_y(),
            z_near: 0.1,
            z_far: 100.0,
        };

        let (camera_configuration, camera_bind_group_layout) = CameraConfiguration::new(&device, &camera, "main");

        let render_pipeline = render_pipeline(
            &device,
            &surface_configuration,
            &diffuse_bind_group_layout,
            &camera_bind_group_layout,
            include_wgsl!("../shaders/shader.wgsl"),
            "shader",
        );

        let depth_texture = Texture::create_depth_texture(
            &device,
            &surface_configuration,
            "depth texture",
        );

        let obj_model = load_model(
            "cube.obj",
            &device,
            &queue,
            &diffuse_bind_group_layout,
        ).await.unwrap();

        Self {
            camera,
            camera_configuration,
            camera_controller,
            depth_texture,
            device,
            instances,
            instance_buffer,
            obj_model,
            queue,
            render_pipeline,
            size,
            surface,
            surface_configuration,
        }
    }
}
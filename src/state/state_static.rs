use wgpu::{Backends, Instance};
use winit::window::Window;

use crate::models::{CameraConfiguration, CameraController, InstanceRaw, ModelVertex, Texture, Vertex};
use crate::resources::load_model;
use crate::State;
use crate::state::initialize::{
    aspect_ratio, configure_surface, create_render_pipeline, diffuse_bind_group_layout,
    get_camera, get_instances, initialize_light, request_adapter, request_device,
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
        let surface_configuration = configure_surface(&adapter, &device, &surface, size);
        let camera_controller = CameraController::new(0.2);
        let (instances, instance_buffer) = get_instances(&device);
        let aspect = aspect_ratio(surface_configuration.width, surface_configuration.height);
        let camera = get_camera(aspect);
        let (camera_configuration, camera_bind_group_layout) = CameraConfiguration::new(&device, &camera, "main");
        let (light, light_bind_group_layout) = initialize_light(&device);
        let diffuse_bind_group_layout = diffuse_bind_group_layout(&device, "diffuse-texture");

        let render_pipeline = create_render_pipeline(
            &device,
            &[
                &diffuse_bind_group_layout,
                &camera_bind_group_layout,
                &light_bind_group_layout,
            ],
            surface_configuration.format,
            Some(Texture::DEPTH_FORMAT),
            &[ModelVertex::desc(), InstanceRaw::desc()],
            include_wgsl!("../shaders/shader.wgsl"),
            "shader",
        );

        let light_pipeline = create_render_pipeline(
            &device,
            &[&camera_bind_group_layout, &light_bind_group_layout],
            surface_configuration.format,
            Some(Texture::DEPTH_FORMAT),
            &[ModelVertex::desc()],
            include_wgsl!("../shaders/light.wgsl"),
            "light",
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
            light,
            light_pipeline,
            obj_model,
            queue,
            render_pipeline,
            size,
            surface,
            surface_configuration,
        }
    }
}

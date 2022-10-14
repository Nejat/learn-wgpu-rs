use std::iter::once;

use wgpu::*;
use wgpu::LoadOp::Clear;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::window::Window;

use crate::controller::CameraController;
use crate::models::{Camera, CameraConfiguration, Geometry, Texture, Vertex};

const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.00759614] },
    Vertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.43041354] },
    Vertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.949397] },
    Vertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.84732914] },
    Vertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.2652641] },
];

const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];

pub struct State {
    camera: Camera,
    camera_configuration: CameraConfiguration,
    camera_controller: CameraController,
    config: SurfaceConfiguration,
    device: Device,
    diffuse_bind_group: BindGroup,
    geometry: Geometry,
    queue: Queue,
    render_pipeline: RenderPipeline,
    pub size: PhysicalSize<u32>,
    surface: Surface,
}

impl State {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = request_adapter(&instance, &surface).await;
        let (device, queue) = request_device(&adapter).await;
        let config = configure_surface(&adapter, &device, &surface, size);
        let (diffuse_bind_group, diffuse_bind_group_layout) = diffuse_texture(&device, &queue, include_bytes!("assets/happy-tree.png"), "happy-tree");
        let geometry = Geometry::new(&device, VERTICES, INDICES);

        let camera = Camera {
            // position the camera one unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, 1.0, 2.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let (camera_configuration, camera_bind_group_layout) = CameraConfiguration::new(&device, &camera, "main");
        let render_pipeline = render_pipeline(
            &device,
            &config,
            &diffuse_bind_group_layout,
            &camera_bind_group_layout,
            include_wgsl!("shader.wgsl"),
            "shader",
        );
        let camera_controller = CameraController::new(0.2);

        Self {
            camera,
            camera_configuration,
            camera_controller,
            config,
            device,
            diffuse_bind_group,
            geometry,
            queue,
            render_pipeline,
            size,
            surface,
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("render encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: Clear(Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            let geometry = &self.geometry;

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_configuration.bind_group, &[]);
            render_pass.set_vertex_buffer(0, geometry.vertex_buffer.slice(..));
            render_pass.set_index_buffer(geometry.index_buffer.slice(..), IndexFormat::Uint16);
            render_pass.draw_indexed(0..geometry.num_indices, 0, 0..1);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(once(encoder.finish()));

        output.present();

        Ok(())
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_configuration.uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(&self.camera_configuration.buffer, 0, bytemuck::cast_slice(&[self.camera_configuration.uniform]));
    }
}

fn configure_surface(
    adapter: &Adapter,
    device: &Device,
    surface: &Surface,
    size: PhysicalSize<u32>,
) -> SurfaceConfiguration {
    let config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_supported_formats(adapter)[0],
        width: size.width,
        height: size.height,
        present_mode: PresentMode::Fifo,
    };

    surface.configure(device, &config);

    config
}

fn diffuse_texture(
    device: &Device,
    queue: &Queue,
    diffuse_bytes: &[u8],
    label: &str,
) -> (BindGroup, BindGroupLayout) {
    let diffuse_texture = Texture::from_bytes(device, queue, diffuse_bytes, label).unwrap();

    let texture_bind_group_layout =
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    // This should match the filterable field of the
                    // corresponding Texture entry above.
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some(&format!("{label} - texture bind group layout")),
        });

    let texture_bind_group = device.create_bind_group(
        &BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&diffuse_texture.view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&diffuse_texture.sampler),
                }
            ],
            label: Some(&format!("{label} - diffuse bind group")),
        }
    );

    (texture_bind_group, texture_bind_group_layout)
}

fn render_pipeline(
    device: &Device,
    config: &SurfaceConfiguration,
    bind_group_layout: &BindGroupLayout,
    camera_bind_group_layout: &BindGroupLayout,
    shader: ShaderModuleDescriptor,
    label: &str,
) -> RenderPipeline {
    let shader = device.create_shader_module(shader);

    let render_pipeline_layout =
        device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(&format!("{label} - render pipeline layout")),
            bind_group_layouts: &[
                bind_group_layout,
                camera_bind_group_layout
            ],
            push_constant_ranges: &[],
        });

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some(&format!("{label} - render pipeline")),
        layout: Some(&render_pipeline_layout),
        vertex: VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[
                Vertex::desc()
            ],
        },
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(ColorTargetState {
                format: config.format,
                blend: Some(BlendState::REPLACE),
                write_mask: ColorWrites::ALL,
            })],
        }),
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: Some(Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: None,
        multisample: MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}

async fn request_adapter(instance: &Instance, surface: &Surface) -> Adapter {
    instance.request_adapter(
        &RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            compatible_surface: Some(surface),
            force_fallback_adapter: false,
        },
    ).await.unwrap()

    // let adapter = instance
    //     .enumerate_adapters(Backends::all())
    //     .filter(|adapter| {
    //         // Check if this adapter supports our surface
    //         surface.get_preferred_format(&adapter).is_some()
    //     })
    //     .next()
    //     .unwrap();
}

async fn request_device(adapter: &Adapter) -> (Device, Queue) {
    adapter.request_device(
        &DeviceDescriptor {
            features: Features::empty(),
            // WebGL doesn't support all of wgpu's features, so if
            // we're building for the web we'll have to disable some.
            limits: if cfg!(target_arch = "wasm32") {
                Limits::downlevel_webgl2_defaults()
            } else {
                Limits::default()
            },
            label: None,
        },
        None, // Trace path
    ).await.unwrap()
}

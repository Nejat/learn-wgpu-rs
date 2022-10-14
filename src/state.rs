use std::iter::once;

use wgpu::{
    Backends, BlendState, Buffer, BufferUsages, Color, ColorTargetState, ColorWrites,
    CommandEncoderDescriptor, Device, DeviceDescriptor, Face, Features, FragmentState,
    FrontFace, IndexFormat, Instance, Limits, MultisampleState, Operations,
    PipelineLayoutDescriptor, PolygonMode, PowerPreference, PresentMode, PrimitiveState,
    PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, Surface, SurfaceConfiguration, SurfaceError,
    TextureUsages, TextureViewDescriptor, VertexState,
};
use wgpu::LoadOp::Clear;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::window::Window;

use crate::models::Vertex;

const VERTICES0: &[Vertex] = &[
    Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
    Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
    Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
];

const INDICES0: &[u16] = &[0, 1, 2];


const VERTICES1: &[Vertex] = &[
    Vertex { position: [-0.0868241, 0.49240386, 0.0], color: [1.0, 0.0, 0.0] }, // A
    Vertex { position: [-0.49513406, 0.06958647, 0.0], color: [1.0, 0.5, 0.0] }, // B
    Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [1.0, 1.0, 0.0] }, // C
    Vertex { position: [0.35966998, -0.3473291, 0.0], color: [0.0, 1.0, 0.0] }, // D
    Vertex { position: [0.44147372, 0.2347359, 0.0], color: [0.0, 0.0, 1.0] }, // E
];

const INDICES1: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];

const VERTICES2: &[Vertex] = &[
    Vertex { position: [0.00, 0.75, 0.00], color: [0.7, 0.0, 1.0] }, // A
    Vertex { position: [-0.21, 0.28, 0.00], color: [0.7, 0.0, 1.0] }, // B
    Vertex { position: [-0.71, 0.23, 0.00], color: [0.7, 0.0, 1.0] }, // C
    Vertex { position: [-0.33, -0.11, 0.00], color: [0.7, 0.0, 1.0] }, // D
    Vertex { position: [-0.44, -0.61, 0.00], color: [0.7, 0.0, 1.0] }, // D
    Vertex { position: [0.00, -0.35, 0.00], color: [0.7, 0.0, 1.0] }, // A
    Vertex { position: [0.44, -0.61, 0.00], color: [0.7, 0.0, 1.0] }, // B
    Vertex { position: [0.33, -0.11, 0.00], color: [0.7, 0.0, 1.0] }, // C
    Vertex { position: [0.71, 0.23, 0.00], color: [0.7, 0.0, 1.0] }, // D
    Vertex { position: [0.21, 0.28, 0.00], color: [0.7, 0.0, 1.0] }, // D
    Vertex { position: [0.00, 0.00, 0.00], color: [1.0, 0.0, 0.8] }, // D
];

const INDICES2: &[u16] = &[
    0, 1, 10,
    1, 2, 10,
    2, 3, 10,
    3, 4, 10,
    4, 5, 10,
    5, 6, 10,
    6, 7, 10,
    7, 8, 10,
    8, 9, 10,
    9, 0, 10,
];

pub struct Geometry {
    index_buffer: Buffer,
    num_indices: u32,
    vertex_buffer: Buffer,
}

pub struct State {
    config: SurfaceConfiguration,
    device: Device,
    geometry: Vec<Geometry>,
    geometry_index: usize,
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

        let adapter = instance.request_adapter(
            &RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        // let adapter = instance
        //     .enumerate_adapters(Backends::all())
        //     .filter(|adapter| {
        //         // Check if this adapter supports our surface
        //         surface.get_preferred_format(&adapter).is_some()
        //     })
        //     .next()
        //     .unwrap();

        let (device, queue) = adapter.request_device(
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
        ).await.unwrap();

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
        };

        surface.configure(&device, &config);

        let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
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
        });

        let geometry = vec![
            create_geometry(&device, VERTICES0, INDICES0),
            create_geometry(&device, VERTICES1, INDICES1),
            create_geometry(&device, VERTICES2, INDICES2),
        ];

        Self {
            config,
            device,
            geometry,
            geometry_index: 0,
            queue,
            render_pipeline,
            size,
            surface,
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        if let WindowEvent::KeyboardInput {
            input:
            KeyboardInput {
                state: ElementState::Pressed,
                virtual_keycode: Some(VirtualKeyCode::Space),
                ..
            },
            ..
        } = event {
            self.geometry_index = (self.geometry_index + 1) % self.geometry.len();

            true
        } else {
            false
        }
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
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

            let geometry = self.geometry.get(self.geometry_index).unwrap();

            render_pass.set_pipeline(&self.render_pipeline);
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

    pub fn update(&mut self) {}
}

fn create_geometry(device: &Device, vertices: &[Vertex], indices: &[u16]) -> Geometry {
    let vertex_buffer = device.create_buffer_init(
        &BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: BufferUsages::VERTEX,
        }
    );

    let index_buffer = device.create_buffer_init(
        &BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: BufferUsages::INDEX,
        }
    );

    Geometry {
        num_indices: indices.len() as u32,
        vertex_buffer,
        index_buffer,
    }
}
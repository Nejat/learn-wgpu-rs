use std::iter::once;

use wgpu::{Backends, BlendState, Color, ColorTargetState, ColorWrites, CommandEncoderDescriptor, Device, DeviceDescriptor, Face, Features, FragmentState, FrontFace, Instance, Limits, MultisampleState, Operations, PipelineLayout, PipelineLayoutDescriptor, PolygonMode, PowerPreference, PresentMode, PrimitiveState, PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, ShaderModule, Surface, SurfaceConfiguration, SurfaceError, TextureUsages, TextureViewDescriptor, VertexState};
use wgpu::LoadOp::Clear;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::window::Window;

pub struct State {
    surface: Surface,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    render_pipeline: RenderPipeline,
    render_pipeline_alt: RenderPipeline,
    pub size: PhysicalSize<u32>,
    use_alt: bool,
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

        let render_pipeline = create_render_pipeline(
            &device,
            &config,
            &shader,
            &render_pipeline_layout,
            "Render Pipeline",
        );

        let shader = device.create_shader_module(include_wgsl!("shader_challenge.wgsl"));

        let render_pipeline_alt = create_render_pipeline(
            &device,
            &config,
            &shader,
            &render_pipeline_layout,
            "Render Pipeline Challenge",
        );

        Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            render_pipeline_alt,
            size,
            use_alt: false,
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
            self.use_alt = !self.use_alt;

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

            render_pass.set_pipeline(self.get_render_pipeline()); // 2.
            render_pass.draw(0..3, 0..1); // 3.
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

    fn get_render_pipeline(&self) -> &RenderPipeline {
        if self.use_alt {
            &self.render_pipeline_alt
        } else {
            &self.render_pipeline
        }
    }
}

fn create_render_pipeline(
    device: &Device,
    config: &SurfaceConfiguration,
    shader: &ShaderModule,
    layout: &PipelineLayout,
    label: &str,
) -> RenderPipeline {
    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(&layout),
        vertex: VertexState {
            module: &shader,
            entry_point: "vs_main", // 1.
            buffers: &[], // 2.
        },
        fragment: Some(FragmentState { // 3.
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(ColorTargetState { // 4.
                format: config.format,
                blend: Some(BlendState::REPLACE),
                write_mask: ColorWrites::ALL,
            })],
        }),
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList, // 1.
            strip_index_format: None,
            front_face: FrontFace::Ccw, // 2.
            cull_mode: Some(Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: None, // 1.
        multisample: MultisampleState {
            count: 1, // 2.
            mask: !0, // 3.
            alpha_to_coverage_enabled: false, // 4.
        },
        multiview: None, // 5.
    })
}

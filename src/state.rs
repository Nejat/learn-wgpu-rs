use std::iter::once;

use wgpu::{
    Backends, Color, CommandEncoderDescriptor, Device, DeviceDescriptor,
    Features, Instance, Limits, Operations, PowerPreference, PresentMode,
    Queue, RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions,
    Surface, SurfaceConfiguration, SurfaceError, TextureUsages, TextureViewDescriptor,
};
use wgpu::LoadOp::Clear;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, ModifiersState, MouseButton, WindowEvent};
use winit::window::Window;

pub struct State {
    clear_color: Color,
    config: SurfaceConfiguration,
    cursor_position: Option<PhysicalPosition<f64>>,
    device: Device,
    kb_state: ModifiersState,
    mouse_input: Option<MouseButton>,
    queue: Queue,
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

        let clear_color = Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };

        Self {
            clear_color,
            config,
            cursor_position: None,
            device,
            kb_state: ModifiersState::default(),
            mouse_input: None,
            queue,
            size,
            surface,
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } =>
                self.cursor_position = Some(*position),
            WindowEvent::ModifiersChanged(modified_state) =>
                self.kb_state = *modified_state,
            WindowEvent::MouseInput { state: mouse_state, button, .. } =>
                if *mouse_state == ElementState::Pressed {
                    self.mouse_input = Some(*button);
                } else {
                    self.mouse_input = None
                },
            // WindowEvent::MouseWheel { delta, phase, .. } => {}
            _ => return false
        }

        true
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let _render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: Clear(self.clear_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
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
        if let Some(PhysicalPosition { x, y }) = self.cursor_position {
            self.clear_color = Color {
                r: x / self.size.width as f64,
                g: (x + y) / (self.size.width + self.size.height) as f64,
                b: y / self.size.height as f64,
                a: 1.0,
            }
        }

        self.cursor_position = None;
    }
}

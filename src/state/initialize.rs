use bytemuck::cast_slice;
use cgmath::{Deg, InnerSpace, Quaternion, Rotation3, Vector3, Zero};
#[allow(clippy::wildcard_imports)]
use wgpu::*;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use winit::dpi::PhysicalSize;

use crate::models::{Instance as MeshInstance, InstanceRaw, Instances, Texture, Vertex};

const NUM_INSTANCES_PER_ROW: u32 = 10;
#[allow(clippy::cast_precision_loss)]
const INSTANCE_DISPLACEMENT: Vector3<f32> = Vector3::new(NUM_INSTANCES_PER_ROW as f32 * 0.5, 0.0, NUM_INSTANCES_PER_ROW as f32 * 0.5);

pub fn diffuse_texture(
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

pub fn get_instances(device: &Device) -> (Instances, Buffer) {
    let instances: Instances = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
        #[allow(clippy::cast_precision_loss)]
        (0..NUM_INSTANCES_PER_ROW).map(move |x| {
            let position = Vector3 { x: x as f32, y: 0.0, z: z as f32 } - INSTANCE_DISPLACEMENT;

            let rotation = if position.is_zero() {
                // this is needed so an object at (0, 0, 0) won't get scaled to zero
                // as Quaternions can effect scale if they're not created correctly
                Quaternion::from_axis_angle(Vector3::unit_z(), Deg(0.0))
            } else {
                Quaternion::from_axis_angle(position.normalize(), Deg(45.0))
            };

            MeshInstance {
                position,
                rotation,
            }
        })
    }).collect::<Vec<_>>().into();

    let instance_data = instances.get_raw();

    let buffer = device.create_buffer_init(
        &BufferInitDescriptor {
            label: Some("instance buffer"),
            contents: cast_slice(&instance_data),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        }
    );

    (instances, buffer)
}

pub fn render_pipeline(
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
                Vertex::desc(),
                InstanceRaw::desc(),
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

pub async fn request_adapter(instance: &Instance, surface: &Surface) -> Adapter {
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

pub async fn request_device(adapter: &Adapter) -> (Device, Queue) {
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

pub fn surface_configuration(
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

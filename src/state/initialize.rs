use bytemuck::cast_slice;
use cgmath::{Deg, InnerSpace, Quaternion, Rotation3, Vector3, Zero};
#[allow(clippy::wildcard_imports)]
use wgpu::*;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use winit::dpi::PhysicalSize;

use crate::models::{Instance as MeshInstance, InstanceRaw, Light, LightUniform};

const NUM_INSTANCES_PER_ROW: u32 = 10;
const SPACE_BETWEEN: f32 = 3.0;

pub fn configure_surface(
    adapter: &Adapter,
    device: &Device,
    surface: &Surface,
    size: PhysicalSize<u32>,
) -> SurfaceConfiguration {
    let configuration = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_supported_formats(adapter)[0],
        width: size.width,
        height: size.height,
        present_mode: PresentMode::Fifo,
        alpha_mode: CompositeAlphaMode::Auto,
    };

    surface.configure(device, &configuration);

    configuration
}

pub fn create_render_pipeline(
    device: &Device,
    bind_group_layouts: &[&BindGroupLayout],
    format: TextureFormat,
    depth_format: Option<TextureFormat>,
    vertex_layouts: &[VertexBufferLayout],
    shader: ShaderModuleDescriptor,
    label: &str,
) -> RenderPipeline {
    let shader = device.create_shader_module(shader);

    let render_pipeline_layout =
        device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(&format!("{label} - pipeline layout")),
            bind_group_layouts,
            push_constant_ranges: &[],
        });

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some(&format!("{label} - render pipeline")),
        layout: Some(&render_pipeline_layout),
        vertex: VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: vertex_layouts,
        },
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(ColorTargetState {
                format,
                blend: Some(BlendState {
                    alpha: BlendComponent::REPLACE,
                    color: BlendComponent::REPLACE,
                }),
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
        depth_stencil: depth_format.map(|format| DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: CompareFunction::Less,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        }),
        multisample: MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}

pub fn diffuse_bind_group_layout(
    device: &Device,
    label: &str,
) -> BindGroupLayout {
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
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    multisampled: false,
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 3,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
        ],
        label: Some(&format!("{label} - bind group layout")),
    })
}

pub fn get_instances(device: &Device) -> (Vec<MeshInstance>, Buffer) {
    let instances = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
        #[allow(clippy::cast_precision_loss)]
        (0..NUM_INSTANCES_PER_ROW).map(move |x| {
            let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
            let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

            let position = Vector3 { x, y: 0.0, z };

            let rotation = if position.is_zero() {
                Quaternion::from_axis_angle(Vector3::unit_z(), Deg(0.0))
            } else {
                Quaternion::from_axis_angle(position.normalize(), Deg(45.0))
            };

            // let rotation = Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), Deg(180.0));

            MeshInstance { position, rotation }
        })
    }).collect::<Vec<_>>();

    let instance_data = instances.iter().map(Into::into).collect::<Vec<InstanceRaw>>();
    let instance_buffer = device.create_buffer_init(
        &BufferInitDescriptor {
            label: Some("instance buffer"),
            contents: cast_slice(&instance_data),
            usage: BufferUsages::VERTEX,
        }
    );

    (instances, instance_buffer)
}

pub fn initialize_light(device: &Device) -> (Light, BindGroupLayout) {
    let light_uniform = LightUniform::new([2.0, 2.0, 2.0], [1.0, 1.0, 1.0]);

    // We'll want to update our lights position, so we use COPY_DST
    let light_buffer = device.create_buffer_init(
        &BufferInitDescriptor {
            label: Some("Light VB"),
            contents: cast_slice(&[light_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        }
    );

    let light_bind_group_layout =
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: None,
        });

    let light_bind_group = device.create_bind_group(&BindGroupDescriptor {
        layout: &light_bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: light_buffer.as_entire_binding(),
        }],
        label: None,
    });

    let light = Light {
        bind_group: light_bind_group,
        buffer: light_buffer,
        uniform: light_uniform,
    };

    (light, light_bind_group_layout)
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

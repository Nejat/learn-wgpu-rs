use bytemuck::cast_slice;
#[allow(clippy::wildcard_imports)]
use wgpu::*;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

use crate::models::{Camera, CameraProjection};
use crate::models::CameraUniform;

pub struct CameraConfiguration {
    pub bind_group: BindGroup,
    pub buffer: Buffer,
    pub uniform: CameraUniform,
}

impl CameraConfiguration {
    pub fn new(
        device: &Device,
        camera: &Camera,
        projection: &CameraProjection,
        label: &str,
    ) -> (Self, BindGroupLayout) {
        let mut uniform = CameraUniform::new();

        uniform.update_view_proj(camera, projection);

        let buffer = device.create_buffer_init(
            &BufferInitDescriptor {
                label: Some(&format!("{label} - camera buffer")),
                contents: cast_slice(&[uniform]),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            }
        );

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some(&format!("{label} - camera bind group layout")),
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ],
            label: Some(&format!("{label} - camera bind group")),
        });

        let configuration = Self { bind_group, buffer, uniform };

        (configuration, bind_group_layout)
    }
}
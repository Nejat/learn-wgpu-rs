use bytemuck::cast_slice;
#[allow(clippy::wildcard_imports)]
use wgpu::*;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

use crate::models::{Camera, CameraUniform};

pub struct CameraConfiguration {
    pub uniform: CameraUniform,
    pub buffer: Buffer,
    pub bind_group: BindGroup,
}

impl CameraConfiguration {
    pub fn new(device: &Device, camera: &Camera, label: &str) -> (Self, BindGroupLayout) {
        let mut camera_uniform = CameraUniform::new();

        camera_uniform.update_view_proj(camera);

        let camera_buffer = device.create_buffer_init(
            &BufferInitDescriptor {
                label: Some(&format!("{label} - camera buffer")),
                contents: cast_slice(&[camera_uniform]),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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

        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some(&format!("{label} - camera bind group")),
        });

        let configuration = Self {
            uniform: camera_uniform,
            buffer: camera_buffer,
            bind_group: camera_bind_group,
        };

        (configuration, camera_bind_group_layout)
    }
}
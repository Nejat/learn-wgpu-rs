use bytemuck::cast_slice;
use wgpu::{Buffer, BufferUsages, Device};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

use crate::models::vertex::Vertex;

pub struct Geometry {
    pub num_indices: u32,
    pub index_buffer: Buffer,
    pub vertex_buffer: Buffer,
}

impl Geometry {
    pub fn new(device: &Device, vertices: &[Vertex], indices: &[u16]) -> Self {
        let vertex_buffer = device.create_buffer_init(
            &BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: cast_slice(vertices),
                usage: BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: cast_slice(indices),
                usage: BufferUsages::INDEX,
            }
        );

        Self {
            num_indices: indices.len() as u32,
            index_buffer,
            vertex_buffer,
        }
    }
}
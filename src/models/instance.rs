use std::mem::size_of;

use cgmath::{Deg, Matrix4, Quaternion, Rotation3, Vector3};
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

pub struct Instance {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
}

pub struct Instances(Vec<Instance>);

impl From<Vec<Instance>> for Instances {
    fn from(src: Vec<Instance>) -> Self {
        Self(src)
    }
}

impl Instances {
    pub fn get_raw(&self) -> Vec<InstanceRaw> {
        self.0.iter().map(Into::into).collect::<Vec<_>>()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn update(&mut self) {
        for instance in &mut self.0 {
            let rotation = instance.rotation;
            let amount = Quaternion::from_angle_y(Deg(6.0));

            instance.rotation = rotation * amount;
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl From<&Instance> for InstanceRaw {
    fn from(src: &Instance) -> Self {
        Self {
            model: (Matrix4::from_translation(src.position) * Matrix4::from(src.rotation)).into(),
        }
    }
}

impl InstanceRaw {
    pub const fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: size_of::<Self>() as BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: VertexStepMode::Instance,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
                    shader_location: 5,
                    format: VertexFormat::Float32x4,
                },
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in
                // the shader.
                VertexAttribute {
                    offset: size_of::<[f32; 4]>() as BufferAddress,
                    shader_location: 6,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: size_of::<[f32; 8]>() as BufferAddress,
                    shader_location: 7,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: size_of::<[f32; 12]>() as BufferAddress,
                    shader_location: 8,
                    format: VertexFormat::Float32x4,
                },
            ],
        }
    }
}

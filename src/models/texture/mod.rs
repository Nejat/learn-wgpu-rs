use wgpu::{Sampler, TextureView};

mod texture_static;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: TextureView,
    pub sampler: Sampler,
}

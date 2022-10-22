#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::fs::read;
use std::io::{BufReader, Cursor};
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

use bytemuck::cast_slice;
use tobj::{load_mtl_buf, load_obj_buf_async, LoadOptions};
use wgpu::{BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, BufferUsages, Device, Queue};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

use crate::models::{Material, Mesh, Model, ModelVertex, Texture};

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();

    let base = reqwest::Url::parse(&format!(
        "{}/{}/",
        location.origin().unwrap(),
        option_env!("RES_PATH").unwrap_or("res"),
    )).unwrap();

    base.join(file_name).unwrap()
}

#[cfg_attr(not(target_arch = "wasm32"), allow(clippy::unused_async))]
#[cfg_attr(target_arch = "wasm32", allow(clippy::future_not_send))] // todo: ???
pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let txt = reqwest::get(url)
                .await?
                .text()
                .await?;
        } else {
            let path = Path::new(env!("OUT_DIR"))
                .join("res")
                .join(file_name);
            let txt = fs::read_to_string(path)?;
        }
    }

    Ok(txt)
}

#[cfg_attr(not(target_arch = "wasm32"), allow(clippy::unused_async))]
#[cfg_attr(target_arch = "wasm32", allow(clippy::future_not_send))] // todo: ???
pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let data = reqwest::get(url).await?.bytes().await?.to_vec();
        } else {
            let path = Path::new(env!("OUT_DIR"))
                .join("res")
                .join(file_name);

            let data = read(path)?;
        }
    }

    Ok(data)
}

#[cfg_attr(target_arch = "wasm32", allow(clippy::future_not_send))] // todo: ???
pub async fn load_model(
    file_name: &str,
    device: &Device,
    queue: &Queue,
    layout: &BindGroupLayout,
) -> anyhow::Result<Model> {
    let obj_text = load_string(file_name).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let (models, obj_materials) = load_obj_buf_async(
        &mut obj_reader,
        &LoadOptions {
            triangulate: true,
            single_index: true,
            ..LoadOptions::default()
        },
        |material_path| async move {
            let mat_text = load_string(&material_path).await.unwrap();

            load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    ).await?;

    let mut materials = Vec::new();

    for material in obj_materials? {
        let diffuse_texture = load_texture(&material.diffuse_texture, device, queue).await?;
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&diffuse_texture.view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: None,
        });

        materials.push(Material {
            name: material.name,
            diffuse_texture,
            bind_group,
        });
    }

    let meshes = models
        .into_iter()
        .map(|model| {
            let vertices = (0..model.mesh.positions.len() / 3)
                .map(|idx| ModelVertex {
                    position: [
                        model.mesh.positions[idx * 3],
                        model.mesh.positions[idx * 3 + 1],
                        model.mesh.positions[idx * 3 + 2],
                    ],
                    tex_coords: [model.mesh.texcoords[idx * 2], model.mesh.texcoords[idx * 2 + 1]],
                    normal: [
                        model.mesh.normals[idx * 3],
                        model.mesh.normals[idx * 3 + 1],
                        model.mesh.normals[idx * 3 + 2],
                    ],
                })
                .collect::<Vec<_>>();

            let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", file_name)),
                contents: cast_slice(&vertices),
                usage: BufferUsages::VERTEX,
            });

            let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", file_name)),
                contents: cast_slice(&model.mesh.indices),
                usage: BufferUsages::INDEX,
            });

            #[allow(clippy::cast_possible_truncation)]
            Mesh {
                name: file_name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: model.mesh.indices.len() as u32,
                material: model.mesh.material_id.unwrap_or(0),
            }
        })
        .collect::<Vec<_>>();

    Ok(Model { meshes, materials })
}

#[cfg_attr(target_arch = "wasm32", allow(clippy::future_not_send))] // todo: ???
pub async fn load_texture(
    file_name: &str,
    device: &Device,
    queue: &Queue,
) -> anyhow::Result<Texture> {
    let data = load_binary(file_name).await?;

    Texture::from_bytes(device, queue, &data, file_name)
}


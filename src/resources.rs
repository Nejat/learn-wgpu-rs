#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::fs::read;
use std::io::{BufReader, Cursor};
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

use bytemuck::cast_slice;
use tobj::{load_mtl_buf, load_obj_buf_async, LoadOptions, Material as ObjMaterial, Model as ObjModel};
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
        materials.push(model_to_material(device, queue, layout, material).await?);
    }

    let meshes = models
        .into_iter()
        .map(|model| model_to_mesh(&model, file_name, device))
        .collect::<Vec<_>>();

    Ok(Model { meshes, materials })
}

#[cfg_attr(target_arch = "wasm32", allow(clippy::future_not_send))] // todo: ???
async fn model_to_material(
    device: &Device,
    queue: &Queue,
    layout: &BindGroupLayout,
    material: ObjMaterial,
) -> anyhow::Result<Material> {
    let diffuse_texture = load_texture(&material.diffuse_texture, device, queue).await?;
    let normal_texture = load_texture(&material.normal_texture, device, queue).await?;

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
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::TextureView(&normal_texture.view),
            },
            BindGroupEntry {
                binding: 3,
                resource: BindingResource::Sampler(&normal_texture.sampler),
            },
        ],
        label: None,
    });

    Ok(Material {
        name: material.name,
        diffuse_texture,
        normal_texture,
        bind_group,
    })
}

fn model_to_mesh(model: &ObjModel, file_name: &str, device: &Device) -> Mesh {
    let mut vertices = (0..model.mesh.positions.len() / 3)
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
            // We'll calculate these later
            tangent: [0.0; 3],
            bi_tangent: [0.0; 3],
        })
        .collect::<Vec<_>>();

    let indices = &model.mesh.indices;
    let mut triangles_included = vec![0; vertices.len()];

    // Calculate tangents and bitangets. We're going to
    // use the triangles, so we need to loop through the
    // indices in chunks of 3
    for c in indices.chunks(3) {
        let v0 = vertices[c[0] as usize];
        let v1 = vertices[c[1] as usize];
        let v2 = vertices[c[2] as usize];

        let pos0: cgmath::Vector3<_> = v0.position.into();
        let pos1: cgmath::Vector3<_> = v1.position.into();
        let pos2: cgmath::Vector3<_> = v2.position.into();

        let uv0: cgmath::Vector2<_> = v0.tex_coords.into();
        let uv1: cgmath::Vector2<_> = v1.tex_coords.into();
        let uv2: cgmath::Vector2<_> = v2.tex_coords.into();

        // Calculate the edges of the triangle
        let delta_pos1 = pos1 - pos0;
        let delta_pos2 = pos2 - pos0;

        // This will give us a direction to calculate the
        // tangent and bi_tangent
        let delta_uv1 = uv1 - uv0;
        let delta_uv2 = uv2 - uv0;

        // Solving the following system of equations will
        // give us the tangent and bi_tangent.
        //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
        //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
        // Luckily, the place I found this equation provided
        // the solution!
        let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
        let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
        // We flip the bi_tangent to enable right-handed normal
        // maps with wgpu texture coordinate system
        let bi_tangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

        // We'll use the same tangent/bi_tangent for each vertex in the triangle
        vertices[c[0] as usize].tangent =
            (tangent + cgmath::Vector3::from(vertices[c[0] as usize].tangent)).into();
        vertices[c[1] as usize].tangent =
            (tangent + cgmath::Vector3::from(vertices[c[1] as usize].tangent)).into();
        vertices[c[2] as usize].tangent =
            (tangent + cgmath::Vector3::from(vertices[c[2] as usize].tangent)).into();
        vertices[c[0] as usize].bi_tangent =
            (bi_tangent + cgmath::Vector3::from(vertices[c[0] as usize].bi_tangent)).into();
        vertices[c[1] as usize].bi_tangent =
            (bi_tangent + cgmath::Vector3::from(vertices[c[1] as usize].bi_tangent)).into();
        vertices[c[2] as usize].bi_tangent =
            (bi_tangent + cgmath::Vector3::from(vertices[c[2] as usize].bi_tangent)).into();

        // Used to average the tangents/bi_tangents
        triangles_included[c[0] as usize] += 1;
        triangles_included[c[1] as usize] += 1;
        triangles_included[c[2] as usize] += 1;
    }

    // Average the tangents/bi_tangents
    for (i, n) in triangles_included.into_iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let denom = 1.0 / n as f32;
        let mut v = &mut vertices[i];
        v.tangent = (cgmath::Vector3::from(v.tangent) * denom).into();
        v.bi_tangent = (cgmath::Vector3::from(v.bi_tangent) * denom).into();
    }

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


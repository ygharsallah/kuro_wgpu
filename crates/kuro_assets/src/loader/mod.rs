use anyhow::{Context, Result};
use kuro_core::prelude::{Mat4, MeshData, Vec3, Vertex};
use std::path::Path;

pub(crate) fn parse_gltf<P: AsRef<Path>>(path: P) -> Result<MeshData> {
    let (document, buffers, _images) = gltf::import(path)?;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let scene = document
        .default_scene()
        .or_else(|| document.scenes().next())
        .context("glTF file contains no scenes")?;

    for node in scene.nodes() {
        process_node(&node, Mat4::IDENTITY, &buffers, &mut vertices, &mut indices);
    }

    Ok(MeshData { vertices, indices })
}

fn process_node(
    node: &gltf::Node,
    parent_transform: Mat4,
    buffers: &[gltf::buffer::Data],
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
) {
    let local_transform = Mat4::from_cols_array_2d(&node.transform().matrix());
    let world_transform = parent_transform * local_transform;

    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            let base_vertex = vertices.len() as u32;

            let positions: Vec<[f32; 3]> = reader
                .read_positions()
                .map(|iter| iter.collect())
                .unwrap_or_default();

            let normals: Vec<[f32; 3]> = reader
                .read_normals()
                .map(|iter| iter.collect())
                .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; positions.len()]);

            for (pos, norm) in positions.into_iter().zip(normals) {
                let transformed_pos = world_transform.transform_point3(Vec3::from(pos));
                let transformed_norm = world_transform
                    .transform_vector3(Vec3::from(norm))
                    .normalize_or_zero();

                vertices.push(Vertex {
                    position: transformed_pos.to_array(),
                    normal: transformed_norm.to_array(),
                });
            }

            if let Some(index_reader) = reader.read_indices() {
                for idx in index_reader.into_u32() {
                    indices.push(base_vertex + idx);
                }
            } else {
                for i in 0..(vertices.len() as u32 - base_vertex) {
                    indices.push(base_vertex + i);
                }
            }
        }
    }

    for child in node.children() {
        process_node(&child, world_transform, buffers, vertices, indices);
    }
}

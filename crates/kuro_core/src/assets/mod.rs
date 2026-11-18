use bytemuck::{Pod, Zeroable};
use uuid::Uuid;

/// Handle pointing to a loaded (or currently loading) mesh inside `Storage`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct AssetId(pub Uuid);

impl AssetId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for AssetId {
    fn default() -> Self {
        Self::new()
    }
}

/// GPU-ready vertex layout shared between the glTF loader and the wgpu renderer.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

/// CPU-side geometry parsed from a `.glb` / `.gltf` file.
pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

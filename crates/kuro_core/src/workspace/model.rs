use crate::assets::AssetId;
use crate::math::{Mat4, Quat, Vec3};
use palette::LinSrgba;
use uuid::Uuid;

/// Handle pointing to a specific spawned instance inside `Workspace`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModelId(pub Uuid);

impl ModelId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ModelId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct Model {
    pub asset_id: AssetId,
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub color: LinSrgba,
}

impl Model {
    pub fn transform_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }
}

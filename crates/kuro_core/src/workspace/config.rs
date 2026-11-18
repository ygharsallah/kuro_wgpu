use crate::math::{Quat, Vec3};
use palette::LinSrgba;

#[derive(Clone, Debug)]
pub struct SpawnConfig {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub color: LinSrgba,
}

impl Default for SpawnConfig {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            color: LinSrgba::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}

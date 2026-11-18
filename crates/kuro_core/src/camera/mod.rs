mod config;

use crate::math::{Mat4, Vec3};
pub use config::CameraConfig;

#[derive(Clone, Debug)]
pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov_degrees: f32,
    pub near: f32,
    pub far: f32,
}

impl From<CameraConfig> for Camera {
    fn from(cfg: CameraConfig) -> Self {
        Self {
            position: cfg.position,
            target: cfg.target,
            up: cfg.up,
            fov_degrees: cfg.fov_degrees,
            near: cfg.near,
            far: cfg.far,
        }
    }
}

impl Camera {
    pub fn build_view_projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        let view = Mat4::look_at_rh(self.position, self.target, self.up);
        let proj = Mat4::perspective_rh(
            self.fov_degrees.to_radians(),
            aspect_ratio.max(0.001),
            self.near,
            self.far,
        );
        proj * view
    }
}

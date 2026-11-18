use crate::math::Vec3;

#[derive(Clone, Debug)]
pub struct CameraConfig {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov_degrees: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 5.0, 10.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov_degrees: 45.0,
            near: 0.1,
            far: 1000.0,
        }
    }
}

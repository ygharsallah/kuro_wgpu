mod config;
mod model;

use crate::assets::AssetId;
use crate::camera::{Camera, CameraConfig};
use crate::math::Vec3;
pub use config::SpawnConfig;
pub use model::{Model, ModelId};
use std::collections::HashMap;

pub struct Workspace {
    pub camera: Camera,
    models: HashMap<ModelId, Model>,
}

impl Workspace {
    pub fn new(camera_config: CameraConfig) -> Self {
        Self {
            camera: Camera::from(camera_config),
            models: HashMap::new(),
        }
    }

    pub fn spawn(&mut self, asset_id: AssetId, position: Vec3) -> ModelId {
        self.spawn_with(
            asset_id,
            SpawnConfig {
                position,
                ..Default::default()
            },
        )
    }

    pub fn spawn_with(&mut self, asset_id: AssetId, config: SpawnConfig) -> ModelId {
        let id = ModelId::new();
        self.models.insert(
            id,
            Model {
                asset_id,
                position: config.position,
                rotation: config.rotation,
                scale: config.scale,
                color: config.color,
            },
        );
        id
    }

    pub fn get_model(&self, id: ModelId) -> Option<&Model> {
        self.models.get(&id)
    }

    pub fn get_model_mut(&mut self, id: ModelId) -> Option<&mut Model> {
        self.models.get_mut(&id)
    }

    pub fn remove_model(&mut self, id: ModelId) -> Option<Model> {
        self.models.remove(&id)
    }

    pub fn models(&self) -> impl Iterator<Item = (&ModelId, &Model)> {
        self.models.iter()
    }
}

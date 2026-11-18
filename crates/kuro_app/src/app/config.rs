use kuro_core::prelude::CameraConfig;

/// Multisampling used for the scene color and depth attachments.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Msaa {
    Off,
    #[default]
    X4,
}

impl Msaa {
    pub(crate) fn samples(self) -> u32 {
        match self {
            Self::Off => 1,
            Self::X4 => 4,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub msaa: Msaa,
    pub camera: CameraConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            title: "Kuro App".into(),
            width: 1280,
            height: 720,
            msaa: Msaa::default(),
            camera: CameraConfig::default(),
        }
    }
}

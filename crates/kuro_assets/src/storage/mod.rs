use crate::loader::parse_gltf;
use anyhow::{Context, Result};
use kuro_core::prelude::{AssetId, MeshData};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

pub struct Storage {
    pending: HashSet<AssetId>,
    sender: Sender<(AssetId, Result<MeshData>)>,
    receiver: Receiver<(AssetId, Result<MeshData>)>,
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}

impl Storage {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            pending: HashSet::new(),
            sender,
            receiver,
        }
    }

    /// Queues a `.glb` or `.gltf` file to load on a background OS thread and returns its `AssetId` immediately.
    pub fn load_gltf<P: Into<PathBuf>>(&mut self, path: P) -> AssetId {
        let id = AssetId::new();
        let path_buf = path.into();
        let tx = self.sender.clone();

        self.pending.insert(id);

        thread::spawn(move || {
            let result =
                parse_gltf(&path_buf).with_context(|| format!("loading {}", path_buf.display()));
            let _ = tx.send((id, result));
        });

        id
    }

    /// Drains any meshes that finished parsing on background threads so `kuro_app` can upload them to VRAM.
    pub fn drain_ready(&mut self) -> Vec<(AssetId, MeshData)> {
        let mut ready = Vec::new();
        while let Ok((id, result)) = self.receiver.try_recv() {
            self.pending.remove(&id);
            match result {
                Ok(mesh_data) => ready.push((id, mesh_data)),
                Err(err) => eprintln!("[kuro_assets] Failed to load glTF: {err:#}"),
            }
        }
        ready
    }

    pub fn is_loading(&self, id: AssetId) -> bool {
        self.pending.contains(&id)
    }
}

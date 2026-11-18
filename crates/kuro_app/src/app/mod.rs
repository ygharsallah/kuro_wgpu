mod config;

pub use config::{AppConfig, Msaa};

use crate::renderer::GpuRenderer;
use kuro_assets::prelude::Storage;
use kuro_core::prelude::Workspace;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

pub struct App {
    config: AppConfig,
    pub storage: Storage,
    pub workspace: Workspace,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        let workspace = Workspace::new(config.camera.clone());
        Self {
            config,
            storage: Storage::new(),
            workspace,
        }
    }

    pub fn run<F>(self, update: F)
    where
        F: FnMut(&mut Workspace, f32) + 'static,
    {
        let event_loop = EventLoop::new().expect("Failed to create winit event loop");
        event_loop.set_control_flow(ControlFlow::Poll);

        let mut runner = AppRunner {
            config: self.config,
            storage: self.storage,
            workspace: self.workspace,
            update,
            window: None,
            renderer: None,
            last_frame: Instant::now(),
        };

        event_loop.run_app(&mut runner).expect("Event loop error");
    }
}

struct AppRunner<F> {
    config: AppConfig,
    storage: Storage,
    workspace: Workspace,
    update: F,
    window: Option<Arc<Window>>,
    renderer: Option<GpuRenderer>,
    last_frame: Instant,
}

impl<F: FnMut(&mut Workspace, f32)> ApplicationHandler for AppRunner<F> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = Window::default_attributes()
                .with_title(&self.config.title)
                .with_inner_size(LogicalSize::new(self.config.width, self.config.height));

            let window = Arc::new(
                event_loop
                    .create_window(attrs)
                    .expect("Failed to create window"),
            );
            let renderer =
                match pollster::block_on(GpuRenderer::new(window.clone(), self.config.msaa)) {
                    Ok(renderer) => renderer,
                    Err(error) => {
                        eprintln!("Failed to initialize renderer: {error:#}");
                        event_loop.exit();
                        return;
                    }
                };
            self.window = Some(window);
            self.renderer = Some(renderer);
            self.last_frame = Instant::now();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = (now - self.last_frame).as_secs_f32().min(0.1);
                self.last_frame = now;

                if let Some(renderer) = &mut self.renderer {
                    for (asset_id, mesh_data) in self.storage.drain_ready() {
                        renderer.upload_mesh(asset_id, &mesh_data);
                    }
                    (self.update)(&mut self.workspace, dt);
                    if let Err(error) = renderer.render(&self.workspace) {
                        eprintln!("Failed to render frame: {error:#}");
                        event_loop.exit();
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

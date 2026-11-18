use anyhow::{Context, Result, ensure};
mod mesh;

use crate::app::Msaa;

use bytemuck::{Pod, Zeroable};
use kuro_core::prelude::{AssetId, Mat4, MeshData, Vertex, Workspace};
use mesh::GpuMesh;
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct ModelUniform {
    model_matrix: [[f32; 4]; 4],
    color: [f32; 4],
}

pub(crate) struct GpuRenderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    model_bind_group_layout: wgpu::BindGroupLayout,
    msaa: Msaa,
    msaa_view: Option<wgpu::TextureView>,
    depth_view: wgpu::TextureView,
    meshes: HashMap<AssetId, GpuMesh>,
}

impl GpuRenderer {
    pub async fn new(window: Arc<Window>, msaa: Msaa) -> Result<Self> {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window)
            .context("Failed to create window surface")?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .context("Failed to find GPU adapter")?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .context("Failed to create GPU device")?;

        let caps = surface.get_capabilities(&adapter);
        ensure!(
            !caps.formats.is_empty() && !caps.alpha_modes.is_empty(),
            "Surface has no compatible configuration"
        );
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: caps.formats[0],
            color_space: Default::default(),
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Workspace Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../../../assets/shaders/shader.wgsl").into(),
            ),
        });

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::bytes_of(&CameraUniform {
                view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let model_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Model Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[Some(&camera_layout), Some(&model_bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
                })],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: msaa.samples(),
                ..Default::default()
            },
            multiview_mask: None,
            cache: None,
        });

        let (depth_view, msaa_view) = Self::create_attachments(&device, &config, msaa);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            camera_buffer,
            camera_bind_group,
            model_bind_group_layout,
            msaa,
            msaa_view,
            depth_view,
            meshes: HashMap::new(),
        })
    }

    fn create_attachments(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        msaa: Msaa,
    ) -> (wgpu::TextureView, Option<wgpu::TextureView>) {
        let create_view = |label, format| {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: config.width,
                    height: config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: msaa.samples(),
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            texture.create_view(&wgpu::TextureViewDescriptor::default())
        };
        let color = match msaa {
            Msaa::Off => None,
            Msaa::X4 => Some(create_view("MSAA Color", config.format)),
        };
        (
            create_view("Depth Texture", wgpu::TextureFormat::Depth32Float),
            color,
        )
    }

    pub fn upload_mesh(&mut self, id: AssetId, data: &MeshData) {
        let gpu_mesh = GpuMesh::from_data(&self.device, data);
        self.meshes.insert(id, gpu_mesh);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            (self.depth_view, self.msaa_view) =
                Self::create_attachments(&self.device, &self.config, self.msaa);
        }
    }

    pub fn render(&mut self, workspace: &Workspace) -> Result<()> {
        let aspect = self.config.width as f32 / self.config.height.max(1) as f32;
        let view_proj = workspace.camera.build_view_projection_matrix(aspect);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::bytes_of(&CameraUniform {
                view_proj: view_proj.to_cols_array_2d(),
            }),
        );

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(f)
            | wgpu::CurrentSurfaceTexture::Suboptimal(f) => f,
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => anyhow::bail!("Render surface was lost"),
            wgpu::CurrentSurfaceTexture::Validation => {
                anyhow::bail!("Surface texture validation failed")
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: self.msaa_view.as_ref().unwrap_or(&view),
                    depth_slice: None,
                    resolve_target: self.msaa_view.as_ref().map(|_| &view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.12,
                            g: 0.14,
                            b: 0.18,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                multiview_mask: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.camera_bind_group, &[]);

            for (_model_id, model) in workspace.models() {
                if let Some(gpu_mesh) = self.meshes.get(&model.asset_id) {
                    let uniform = ModelUniform {
                        model_matrix: model.transform_matrix().to_cols_array_2d(),
                        color: model.color.into(),
                    };
                    let model_buf =
                        self.device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Model Uniform Buffer"),
                                contents: bytemuck::bytes_of(&uniform),
                                usage: wgpu::BufferUsages::UNIFORM,
                            });
                    let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Model Bind Group"),
                        layout: &self.model_bind_group_layout,
                        entries: &[wgpu::BindGroupEntry {
                            binding: 0,
                            resource: model_buf.as_entire_binding(),
                        }],
                    });

                    rpass.set_bind_group(1, &bind_group, &[]);
                    rpass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
                    rpass.set_index_buffer(
                        gpu_mesh.index_buffer.slice(..),
                        wgpu::IndexFormat::Uint32,
                    );
                    rpass.draw_indexed(0..gpu_mesh.index_count, 0, 0..1);
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(frame);
        Ok(())
    }
}

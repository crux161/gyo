use std::sync::Arc;
use std::mem;
use hanga_traits::Runtime;
//use wgpu::util::DeviceExt; 

pub mod pipeline_2d;
pub mod loader;

use pipeline_2d::{SpriteBatch, InstanceRaw};
use loader::ProjectLoader;

pub struct HangaEngine {
    window: Arc<winit::window::Window>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    
    // Pipelines
    render_pipeline: wgpu::RenderPipeline, // Sprites
    sky_pipeline: wgpu::RenderPipeline,    // Sky (NEW)
    
    sprite_batch: SpriteBatch,
}

impl HangaEngine {
    async fn init_wgpu(window: Arc<winit::window::Window>) -> (Arc<wgpu::Device>, Arc<wgpu::Queue>, wgpu::Surface<'static>, wgpu::SurfaceConfiguration) {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).expect("Failed to create surface");
        let surface: wgpu::Surface<'static> = unsafe { mem::transmute(surface) };

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.expect("No suitable GPU adapter found");

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Hanga Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None, 
        ).await.expect("Failed to create device");

        let size = window.inner_size();
        let config = surface.get_default_config(&adapter, size.width, size.height)
            .expect("Surface not supported");
        
        surface.configure(&device, &config);

        (Arc::new(device), Arc::new(queue), surface, config)
    }

    // Helper for the Sky (No Vertex Buffers)
    fn create_sky_pipeline(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::include_wgsl!("sky.wgsl"));
        
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sky Pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[], // No buffers needed!
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE), // Sky overwrites everything
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1, mask: !0, alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        })
    }

    // Helper for Sprites (Loaded Code)
    fn create_pipeline(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, source_code: &str) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Gyosho Loaded Shader"),
            source: wgpu::ShaderSource::Wgsl(source_code.into()),
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sprite Pipeline"),
            layout: None, 
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[InstanceRaw::desc()], 
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING), // Sprites blend over sky
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, 
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1, mask: !0, alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        })
    }

    pub fn get_batch(&mut self) -> &mut SpriteBatch { &mut self.sprite_batch }
    pub fn prepare_frame(&mut self) { self.sprite_batch.prepare(&self.device, &self.queue); }
}

impl Runtime for HangaEngine {
    async fn new(window: Arc<winit::window::Window>, project_bytes: &[u8]) -> anyhow::Result<Self> {
        let (device, queue, surface, config) = Self::init_wgpu(window.clone()).await;
        
        println!("📂 Engine Loading Project...");
        let project = ProjectLoader::load(project_bytes)?;
        
        // Create BOTH pipelines
        let render_pipeline = Self::create_pipeline(&device, &config, &project.source_code);
        let sky_pipeline = Self::create_sky_pipeline(&device, &config);
        
        let sprite_batch = SpriteBatch::new(&device, 10_000); 

        Ok(Self {
            window, device, queue, surface, config,
            render_pipeline,
            sky_pipeline,
            sprite_batch,
        })
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Hanga Render Encoder"),
        });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), // Clear logic still handled here
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // 1. DRAW SKY
            rpass.set_pipeline(&self.sky_pipeline);
            rpass.draw(0..3, 0..1); // Draw 3 vertices (1 triangle)

            // 2. DRAW SPRITES (Rain)
            rpass.set_pipeline(&self.render_pipeline);
            self.sprite_batch.draw(&mut rpass);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

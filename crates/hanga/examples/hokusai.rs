use std::sync::Arc;
use std::time::Instant; // 🌟 NEW: Time tracking
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use hanga::HangaEngine;
use hanga_traits::Runtime;
use rand::Rng;
use gyo_core::{GyoshoFile, Manifest, AssetEntry, AssetKind};
use std::io::Cursor;

// --- DUMMY FILE GENERATOR (UNCHANGED) ---
fn create_dummy_gyo() -> anyhow::Result<Vec<u8>> {
    let shader_code = r#"
    struct VertexOutput {
        @builtin(position) clip_position: vec4<f32>,
        @location(0) uv: vec2<f32>,
        @location(1) color: vec4<f32>,
    };
    struct InstanceInput {
        @location(0) pos: vec3<f32>,
        @location(1) size: vec2<f32>,
        @location(2) uv_rect: vec4<f32>,
        @location(3) color: vec4<f32>,
    };
    @vertex
    fn vs_main(@builtin(vertex_index) in_vertex_index: u32, instance: InstanceInput) -> VertexOutput {
        var corners = array<vec2<f32>, 6>(
            vec2(0.0, 0.0), vec2(0.0, 1.0), vec2(1.0, 1.0),
            vec2(0.0, 0.0), vec2(1.0, 1.0), vec2(1.0, 0.0)
        );
        let local_pos = corners[in_vertex_index];
        let world_x = instance.pos.x + (local_pos.x * instance.size.x);
        let world_y = instance.pos.y + (local_pos.y * instance.size.y);
        
        // Normalize to NDC
        let ndc_x = (world_x / 1280.0) * 2.0 - 1.0;
        let ndc_y = (world_y / 720.0) * -2.0 + 1.0;
        
        var out: VertexOutput;
        out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
        out.uv = local_pos; 
        out.color = instance.color;
        return out;
    }
    @fragment
    fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
        return in.color;
    }
    "#;

    let assets = vec![AssetEntry {
        id: "main.wgsl".to_string(),
        kind: AssetKind::SumiSource,
        offset: 0,
        size: shader_code.len() as u64,
    }];
    
    let manifest = Manifest {
        title: "Generated Rain".to_string(),
        author: "Gyosho Builder".to_string(),
        timestamp: 0,
        assets,
        compute_kernels: vec![],
    };

    let mut buffer = Cursor::new(Vec::new());
    GyoshoFile::write_new(&mut buffer, &manifest, shader_code.as_bytes())?;
    Ok(buffer.into_inner())
}

// --- SIMULATION ---

struct RainDrop {
    x: f32, 
    y: f32, 
    speed: f32, 
    scale: f32,
    // Store original X to allow wrapping without visible seams
    epoch: u32, 
}

struct HokusaiApp {
    window: Option<Arc<Window>>,
    engine: Option<HangaEngine>,
    drops: Vec<RainDrop>,
    start_time: Instant, // 🌟 NEW: The Clock
}

impl ApplicationHandler for HokusaiApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Gyosho: Hokusai Storm (Wind Active)")
                .with_inner_size(winit::dpi::PhysicalSize::new(1280, 720));

            let window = Arc::new(event_loop.create_window(window_attributes).expect("Failed to create window"));
            self.window = Some(window.clone());

            let gyo_bytes = create_dummy_gyo().expect("Failed to create GYO");
            let engine = pollster::block_on(HangaEngine::new(window.clone(), &gyo_bytes))
                .expect("Failed to initialize engine");
            
            // Generate Drops
            let mut rng = rand::rng(); 
            self.drops.reserve(50_000);
            for _ in 0..50_000 {
                self.drops.push(RainDrop {
                    x: rng.random_range(-200.0..1480.0), // Wider spawn area for wind
                    y: rng.random_range(0.0..720.0),
                    speed: rng.random_range(15.0..25.0), // Faster!
                    scale: rng.random_range(0.5..1.2),
                    epoch: 0,
                });
            }
            self.engine = Some(engine);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(engine) = self.engine.as_mut() {
                    engine.resize(size);
                    if let Some(window) = self.window.as_ref() { window.request_redraw(); }
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(engine) = self.engine.as_mut() {
                    // 🌟 1. CALCULATE WIND
                    let elapsed = self.start_time.elapsed().as_secs_f32();
                    // Sine wave: oscillating left and right over 5 seconds
                    let wind_force = (elapsed * 2.0).sin() * 5.0; 
                    
                    // 🌟 2. UPDATE PHYSICS
                    for drop in &mut self.drops {
                        drop.y += drop.speed;
                        drop.x += wind_force * drop.scale; // Parallax wind!

                        // Wrapping Logic
                        if drop.y > 720.0 {
                            drop.y = -50.0;
                            // Randomize X slightly on respawn to break patterns
                            drop.x = rand::rng().random_range(-200.0..1480.0);
                        }
                    }

                    // 🌟 3. BUILD BATCH
                    let batch = engine.get_batch();
                    batch.clear();
                    
                    for drop in &self.drops {
                        // Slant the rain based on wind (simple skew effect)
                        let slant = wind_force * 2.0;
                        
                        batch.push(hanga::pipeline_2d::InstanceRaw {
                            pos: [drop.x, drop.y, 0.0],
                            // We stretch the width slightly when it moves fast
                            size: [2.0 * drop.scale, 25.0 * drop.scale], 
                            uv_rect: [0.0, 0.0, 1.0, 1.0],
                            // Color varies slightly by speed (faster = lighter)
                            color: [0.6, 0.7 + (drop.speed/100.0), 1.0, 0.4], 
                        });
                    }

                    engine.prepare_frame();
                    if let Err(e) = engine.render() { eprintln!("{:?}", e); }
                    if let Some(window) = self.window.as_ref() { window.request_redraw(); }
                }
            }
            _ => {}
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    
    let mut app = HokusaiApp { 
        window: None, 
        engine: None, 
        drops: Vec::new(),
        start_time: Instant::now() // Start the clock
    };
    
    event_loop.run_app(&mut app)?;
    Ok(())
}

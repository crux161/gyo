use winit::{window::Window, event::Event};
use std::sync::Arc;
use anyhow::Result;

pub trait Runtime: 'static + Sized {
    // 🌟 FIX: The second argument MUST be 'project_bytes: &[u8]'
    // NOT 'manifest: &Manifest'
    fn new(window: Arc<Window>, project_bytes: &[u8]) -> impl std::future::Future<Output = Result<Self>> + Send;

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>);
    fn render(&mut self) -> Result<(), wgpu::SurfaceError>;
    fn process_input(&mut self, _event: &Event<()>) -> bool {
        false
    }
}

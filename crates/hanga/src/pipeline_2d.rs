use bytemuck::{Pod, Zeroable};

/// The "Atom" of the 2D engine. 
/// Matches the Buffer Layout in the WGSL Vertex Shader.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct InstanceRaw {
    /// Position (x, y) + Depth/Layer (z)
    pub pos: [f32; 3], 
    /// Size (width, height)
    pub size: [f32; 2],
    /// UV Offset (x, y) and Scale (w, h) for texture atlases
    pub uv_rect: [f32; 4],
    /// Color Tint (RGBA)
    pub color: [f32; 4],
}

impl InstanceRaw {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // Location 0: Pos (vec3)
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Location 1: Size (vec2)
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Location 2: UV Rect (vec4)
                wgpu::VertexAttribute {
                    offset: (mem::size_of::<[f32; 3]>() + mem::size_of::<[f32; 2]>()) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Location 3: Color (vec4)
                wgpu::VertexAttribute {
                    offset: (mem::size_of::<[f32; 3]>() + mem::size_of::<[f32; 2]>() + mem::size_of::<[f32; 4]>()) as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct SpriteBatch {
    instances: Vec<InstanceRaw>,
    instance_buffer: wgpu::Buffer,
    capacity: usize,
}

impl SpriteBatch {
    pub fn new(device: &wgpu::Device, initial_capacity: usize) -> Self {
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sprite Instance Buffer"),
            size: (initial_capacity * std::mem::size_of::<InstanceRaw>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            instances: Vec::with_capacity(initial_capacity),
            instance_buffer,
            capacity: initial_capacity,
        }
    }

    /// Add a sprite to the batch for this frame
    pub fn push(&mut self, instance: InstanceRaw) {
        self.instances.push(instance);
    }

    /// Clear the batch (call at start of frame)
    pub fn clear(&mut self) {
        self.instances.clear();
    }

    /// Uploads data to GPU. Resizes the buffer if we have more sprites than capacity.
    pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.instances.is_empty() { return; }

        // Resize buffer if needed (Double capacity strategy)
        if self.instances.len() > self.capacity {
            self.capacity = self.instances.len().max(self.capacity * 2);
            println!("⚡ SpriteBatch Resizing: New Capacity = {}", self.capacity);
            
            self.instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Sprite Instance Buffer (Resized)"),
                size: (self.capacity * std::mem::size_of::<InstanceRaw>()) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        // Upload data
        queue.write_buffer(
            &self.instance_buffer, 
            0, 
            bytemuck::cast_slice(&self.instances)
        );
    }

    pub fn draw<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>) {
        if self.instances.is_empty() { return; }
        
        // FIX: Bind to Slot 0 (Since we have no Geometry buffer)
        rpass.set_vertex_buffer(0, self.instance_buffer.slice(0..(self.instances.len() * std::mem::size_of::<InstanceRaw>()) as wgpu::BufferAddress));
        
        // Draw 6 vertices (1 Quad) * N Instances
        rpass.draw(0..6, 0..self.instances.len() as u32);
    }
}

// crates/hanga/src/shader.wgsl

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
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    instance: InstanceInput,
) -> VertexOutput {
    // Define a Unit Quad (0,0 to 1,1) using a hardcoded array
    // This avoids needing a separate Vertex Buffer for geometry
    var corners = array<vec2<f32>, 6>(
        vec2(0.0, 0.0), vec2(0.0, 1.0), vec2(1.0, 1.0),
        vec2(0.0, 0.0), vec2(1.0, 1.0), vec2(1.0, 0.0)
    );
    let local_pos = corners[in_vertex_index];
    
    // Scale and Translate
    let world_x = instance.pos.x + (local_pos.x * instance.size.x);
    let world_y = instance.pos.y + (local_pos.y * instance.size.y);
    
    // Convert to NDC (Normalized Device Coordinates: -1.0 to 1.0)
    // TODO: Pass screen size via Uniforms. Hardcoded to 1280x720 for Prototype.
    let ndc_x = (world_x / 1280.0) * 2.0 - 1.0;
    let ndc_y = (world_y / 720.0) * -2.0 + 1.0; // Flip Y for WGPU

    var out: VertexOutput;
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.uv = local_pos; 
    out.color = instance.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Just return the instance color for now (Flat shading)
    return in.color;
}

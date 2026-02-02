// crates/hanga/src/sky.wgsl

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    // Generate a full-screen triangle from 3 vertices
    // (-1, -1), (3, -1), (-1, 3) covers the screen
    var uv = vec2<f32>(
        f32((in_vertex_index << 1u) & 2u),
        f32(in_vertex_index & 2u)
    );
    var out: VertexOutput;
    out.position = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
    // Invert Y for WGPU texture coordinates if needed, but for sky it matters little
    out.uv = uv;
    return out;
}

// --- FBM NOISE FUNCTIONS ---
// A simple hash function
fn hash(p: vec2<f32>) -> f32 {
    var p2 = fract(p * vec2<f32>(123.34, 456.21));
    p2 = p2 + dot(p2, p2 + 45.32);
    return fract(p2.x * p2.y);
}

// Bilinear value noise
fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    return mix(mix(hash(i + vec2<f32>(0.0,0.0)), 
                   hash(i + vec2<f32>(1.0,0.0)), u.x),
               mix(hash(i + vec2<f32>(0.0,1.0)), 
                   hash(i + vec2<f32>(1.0,1.0)), u.x), u.y);
}

// Fractal Brownian Motion (The "Cloud" look)
fn fbm(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var st = p;
    for (var i = 0; i < 5; i++) {
        value += amplitude * noise(st);
        st *= 2.0;
        amplitude *= 0.5;
    }
    return value;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Basic sky gradient
    let sky_blue = vec3<f32>(0.0, 0.05, 0.2); // Deep blue
    let clouds = vec3<f32>(0.8, 0.8, 0.9);    // White-ish
    
    // Calculate noise based on UV
    // TODO: Pass 'time' uniform to animate this later!
    let n = fbm(in.uv * 3.0);
    
    // Mix sky and clouds
    let color = mix(sky_blue, clouds, n);
    
    return vec4<f32>(color, 1.0);
}

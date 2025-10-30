// Simplified WGSL shader for cube rendering
// This is a basic version - we'll enhance it later

pub const SIMPLE_CUBE_VERTEX_SHADER: &str = r#"
@group(0) @binding(0) var<uniform> perspective: mat4x4<f32>;
@group(0) @binding(1) var<uniform> view: mat4x4<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) instance_position: vec3<f32>,
    @location(2) block_id: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) block_id: u32,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    // Transform position to world space (cube is centered at origin, move to instance position)
    let world_pos = input.position + input.instance_position;
    let world_pos4 = vec4<f32>(world_pos, 1.0);
    output.position = perspective * view * world_pos4;
    output.block_id = input.block_id;
    return output;
}
"#;

pub const SIMPLE_CUBE_FRAGMENT_SHADER: &str = r#"
struct FragmentInput {
    @location(0) block_id: u32,
}

@fragment
fn fs_main(input: FragmentInput) -> @location(0) vec4<f32> {
    // Simple color mapping based on block ID
    // This is a placeholder - will be replaced with texture lookups later
    let colors = array<vec3<f32>, 9>(
        vec3<f32>(0.1, 0.7, 0.1),  // GRASS - green
        vec3<f32>(0.6, 0.4, 0.2),  // DIRT - brown
        vec3<f32>(0.5, 0.5, 0.5),  // COBBELSTONE - gray
        vec3<f32>(0.4, 0.3, 0.2),  // OAKLOG - brown
        vec3<f32>(0.2, 0.6, 0.2),  // OAKLEAVES - green
        vec3<f32>(0.2, 0.5, 0.9), // WATER - blue
        vec3<f32>(0.6, 0.6, 0.6), // STONE - light gray
        vec3<f32>(0.9, 0.8, 0.6), // SAND - beige
        vec3<f32>(0.3, 0.3, 0.3), // SWORD - dark gray
    );
    
    let idx = min(input.block_id, 8u);
    return vec4<f32>(colors[idx], 1.0);
}
"#;

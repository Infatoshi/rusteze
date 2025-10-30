// WGSL shader for cube rendering with texture atlas
// Uses a texture atlas containing all block textures

@group(0) @binding(0) var<uniform> perspective: mat4x4<f32>;
@group(0) @binding(1) var<uniform> view: mat4x4<f32>;
@group(1) @binding(0) var t_diffuse: texture_2d<f32>;
@group(1) @binding(1) var s_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) face: u32,
    @location(3) instance_position: vec3<f32>,
    @location(4) block_id: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) block_id: u32,
    @location(2) face: u32,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    // Transform position to world space (cube vertices are centered at origin)
    let world_pos = input.position + input.instance_position;
    let world_pos4 = vec4<f32>(world_pos, 1.0);
    output.position = perspective * view * world_pos4;
    output.tex_coords = input.tex_coords;
    output.block_id = input.block_id;
    output.face = input.face;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Texture atlas layout:
    // - 16 textures per row
    // - Each block has 3 textures: side (idx 0), top (idx 1), bottom (idx 2)
    // - Texture index = block_id * 3 + face_index
    
    let textures_per_row = 16.0;
    let texture_size = 64.0;
    let atlas_width = textures_per_row * texture_size;
    
    // Calculate which texture slot we're using
    let face_idx = input.face;
    let texture_slot = f32(input.block_id * 3u + face_idx);
    
    // Calculate position in atlas grid
    // row = which row in the grid (vertical position)
    // col = which column in the row (horizontal position)
    let row = floor(texture_slot / textures_per_row);
    let col = texture_slot % textures_per_row;
    
    // Calculate UV offset for this texture in the atlas
    // u is horizontal (x), v is vertical (y)
    let u_offset = col * texture_size / atlas_width;
    let v_offset = row * texture_size / atlas_width;
    let u_scale = texture_size / atlas_width;
    let v_scale = texture_size / atlas_width;
    
    // Transform the face-local UV coordinates (0-1) to atlas coordinates
    let atlas_uv = vec2<f32>(
        u_offset + input.tex_coords.x * u_scale,
        v_offset + input.tex_coords.y * v_scale
    );
    
    // Sample the texture
    return textureSample(t_diffuse, s_diffuse, atlas_uv);
}

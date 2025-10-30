// Note: Removed glium dependency - will be replaced with wgpu rendering

// Vertex shader
// Most basic example with a camera
pub const CUBE_VERTEX_SHADER: &str = r#"
        #version 150

        in vec3 position;
        in mat4 world_matrix;

        // The vertex shader has some passthrough for the fragment shader...

        // Which face of the cube is being passed ?
        in int face;
        flat out int face_s;

        // Index of the block to be used
        in int block_id;
        flat out int block_id_s;

        // Is the cube currently selected
        in int is_selected;
        flat out int is_selected_s;

        // Where is the vertex located on the face ?
        in vec2 tex_coords;
        out vec2 v_tex_coords;

        uniform mat4 perspective;
        uniform mat4 view;

        void main() {
            gl_Position = perspective * view * world_matrix * vec4(position, 1.0);
            v_tex_coords = tex_coords;
            face_s = face;
            block_id_s = block_id;
            is_selected_s = is_selected;
        }
    "#;

// Fragment shader
pub const CUBE_FRAGMENT_SHADER: &str = r#"
        #version 140

        // passed-through the vertex shader
        flat in int face_s;
        flat in int block_id_s;
        flat in int is_selected_s;
        in vec2 v_tex_coords;

        out vec4 color ;

        uniform sampler2DArray textures;
        
        // uniforms for the selected block
        uniform sampler2D selected_texture;
        uniform float selected_intensity;

        void main() {
            // Each block has 3 types of faces
            int idx = block_id_s * 3;

            if (face_s == 5) {
                // bottom
                color = texture(textures, vec3(v_tex_coords, idx + 2));
            } else if (face_s == 4) {
                // top
                color = texture(textures, vec3(v_tex_coords, idx + 1));
            } else {
                // sides
                color = texture(textures, vec3(v_tex_coords, float(idx)));
            }

            if (is_selected_s != 0) {
                color = mix(color, texture(selected_texture, v_tex_coords), selected_intensity);
            }
        }
    "#;

use bytemuck::{Pod, Zeroable};

/// A vertex of a cube
/// The position is expressed into the OpenGL reference frame
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct CubeVertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
    face: u32, // Changed to u32 to avoid padding issues
}

// Note: Removed glium::implement_vertex! macro - will be replaced with wgpu rendering

pub const VERTICES: [CubeVertex; 36] = [
    // Right side
    CubeVertex {
        position: [-0.5, -0.5, -0.5],
        tex_coords: [0.0, 0.0],
        face: 0 as u32,
    },
    CubeVertex {
        position: [0.5, -0.5, -0.5],
        tex_coords: [1.0, 0.0],
        face: 0 as u32,
    },
    CubeVertex {
        position: [0.5, 0.5, -0.5],
        tex_coords: [1.0, 1.0],
        face: 0 as u32,
    },
    CubeVertex {
        position: [0.5, 0.5, -0.5],
        tex_coords: [1.0, 1.0],
        face: 0 as u32,
    },
    CubeVertex {
        position: [-0.5, 0.5, -0.5],
        tex_coords: [0.0, 1.0],
        face: 0 as u32,
    },
    CubeVertex {
        position: [-0.5, -0.5, -0.5],
        tex_coords: [0.0, 0.0],
        face: 0 as u32,
    },
    // Front
    CubeVertex {
        position: [0.5, 0.5, 0.5],
        tex_coords: [1.0, 1.0],
        face: 1 as u32,
    },
    CubeVertex {
        position: [0.5, 0.5, -0.5],
        tex_coords: [0.0, 1.0],
        face: 1 as u32,
    },
    CubeVertex {
        position: [0.5, -0.5, -0.5],
        tex_coords: [0.0, 0.0],
        face: 1 as u32,
    },
    CubeVertex {
        position: [0.5, -0.5, -0.5],
        tex_coords: [0.0, 0.0],
        face: 1 as u32,
    },
    CubeVertex {
        position: [0.5, -0.5, 0.5],
        tex_coords: [1.0, 0.0],
        face: 1 as u32,
    },
    CubeVertex {
        position: [0.5, 0.5, 0.5],
        tex_coords: [1.0, 1.0],
        face: 1 as u32,
    },
    // Left side
    CubeVertex {
        position: [-0.5, -0.5, 0.5],
        tex_coords: [1.0, 0.0],
        face: 2 as u32,
    },
    CubeVertex {
        position: [0.5, -0.5, 0.5],
        tex_coords: [0.0, 0.0],
        face: 2 as u32,
    },
    CubeVertex {
        position: [0.5, 0.5, 0.5],
        tex_coords: [0.0, 1.0],
        face: 2 as u32,
    },
    CubeVertex {
        position: [0.5, 0.5, 0.5],
        tex_coords: [0.0, 1.0],
        face: 2 as u32,
    },
    CubeVertex {
        position: [-0.5, 0.5, 0.5],
        tex_coords: [1.0, 1.0],
        face: 2 as u32,
    },
    CubeVertex {
        position: [-0.5, -0.5, 0.5],
        tex_coords: [1.0, 0.0],
        face: 2 as u32,
    },
    // Back
    CubeVertex {
        position: [-0.5, 0.5, 0.5],
        tex_coords: [0.0, 1.0],
        face: 3 as u32,
    },
    CubeVertex {
        position: [-0.5, 0.5, -0.5],
        tex_coords: [1.0, 1.0],
        face: 3 as u32,
    },
    CubeVertex {
        position: [-0.5, -0.5, -0.5],
        tex_coords: [1.0, 0.0],
        face: 3 as u32,
    },
    CubeVertex {
        position: [-0.5, -0.5, -0.5],
        tex_coords: [1.0, 0.0],
        face: 3 as u32,
    },
    CubeVertex {
        position: [-0.5, -0.5, 0.5],
        tex_coords: [0.0, 0.0],
        face: 3 as u32,
    },
    CubeVertex {
        position: [-0.5, 0.5, 0.5],
        tex_coords: [0.0, 1.0],
        face: 3 as u32,
    },
    // Top
    CubeVertex {
        position: [-0.5, 0.5, -0.5],
        tex_coords: [0.0, 1.0],
        face: 4 as u32,
    },
    CubeVertex {
        position: [0.5, 0.5, -0.5],
        tex_coords: [0.0, 0.0],
        face: 4 as u32,
    },
    CubeVertex {
        position: [0.5, 0.5, 0.5],
        tex_coords: [1.0, 0.0],
        face: 4 as u32,
    },
    CubeVertex {
        position: [0.5, 0.5, 0.5],
        tex_coords: [1.0, 0.0],
        face: 4 as u32,
    },
    CubeVertex {
        position: [-0.5, 0.5, 0.5],
        tex_coords: [1.0, 1.0],
        face: 4 as u32,
    },
    CubeVertex {
        position: [-0.5, 0.5, -0.5],
        tex_coords: [0.0, 1.0],
        face: 4 as u32,
    },
    //  Bottom
    CubeVertex {
        position: [-0.5, -0.5, -0.5],
        tex_coords: [0.0, 1.0],
        face: 5 as u32,
    },
    CubeVertex {
        position: [0.5, -0.5, -0.5],
        tex_coords: [0.0, 0.0],
        face: 5 as u32,
    },
    CubeVertex {
        position: [0.5, -0.5, 0.5],
        tex_coords: [1.0, 0.0],
        face: 5 as u32,
    },
    CubeVertex {
        position: [0.5, -0.5, 0.5],
        tex_coords: [1.0, 0.0],
        face: 5 as u32,
    },
    CubeVertex {
        position: [-0.5, -0.5, 0.5],
        tex_coords: [1.0, 1.0],
        face: 5 as u32,
    },
    CubeVertex {
        position: [-0.5, -0.5, -0.5],
        tex_coords: [0.0, 1.0],
        face: 5 as u32,
    },
];

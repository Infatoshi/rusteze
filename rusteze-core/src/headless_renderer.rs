use crate::camera::perspective_matrix;
use crate::game::player::Player;
use crate::opengl::cube::{CubeVertex, VERTICES};
use crate::world::block_kind::Block;
use crate::world::world::World;
use bytemuck::{Pod, Zeroable};
use image::RgbaImage;
use pollster::block_on;
use std::path::Path;
use std::sync::Arc;
use strum::IntoEnumIterator;
use wgpu::util::DeviceExt;
use wgpu::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Uniforms {
    perspective: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]  // Fields may be used in future shader optimizations
struct InstanceData {
    position: [f32; 3],
    block_id: u32,
}

unsafe impl Pod for InstanceData {}
unsafe impl Zeroable for InstanceData {}

/// UV coordinates for a texture in the atlas
#[derive(Clone, Copy, Debug)]
struct TextureUV {
    /// Top-left corner (normalized 0-1)
    u_min: f32,
    v_min: f32,
    /// Bottom-right corner (normalized 0-1)
    u_max: f32,
    v_max: f32,
}

/// Headless renderer using wgpu for off-screen rendering
pub struct HeadlessRenderer {
    device: Arc<Device>,
    queue: Arc<Queue>,
    render_pipeline: RenderPipeline,
    uniform_bind_group: BindGroup,
    uniform_buffer: Buffer,
    texture_bind_group: BindGroup,
    texture: Texture,
    sampler: Sampler,
    depth_texture: Texture,
    depth_view: TextureView,
    staging_buffer: Buffer,
    width: u32,
    height: u32,
}

impl HeadlessRenderer {
    /// Initialize a new headless renderer
    pub fn new(width: u32, height: u32) -> Self {
        block_on(Self::new_async(width, height))
    }

    async fn new_async(width: u32, height: u32) -> Self {
        // Create instance
        let instance = Instance::new(InstanceDescriptor::default());

        // Create a headless adapter
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find an appropriate adapter");

        // Create device and queue
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        // Load texture atlas
        let (texture, texture_bind_group, _uv_map) = Self::load_texture_atlas(&device, &queue);

        // Create shaders
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Cube Shader"),
            source: ShaderSource::Wgsl(include_str!("shaders.wgsl").into()),
        });

        // Create uniform buffer
        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layout for uniforms
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Uniform Bind Group Layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        // Create bind group for uniforms
        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &uniform_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });

        // Create bind group layout for textures (will be used in shader)
        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            view_dimension: TextureViewDimension::D2,
                            sample_type: TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        // Create vertex buffer layout
        let vertex_buffer_layouts = [
            // Cube vertices
            VertexBufferLayout {
                array_stride: std::mem::size_of::<CubeVertex>() as u64,
                step_mode: VertexStepMode::Vertex,
                attributes: &[
                    VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: VertexFormat::Float32x3,
                    },
                    VertexAttribute {
                        offset: 12, // sizeof(f32) * 3
                        shader_location: 1,
                        format: VertexFormat::Float32x2,
                    },
                    VertexAttribute {
                        offset: 20, // sizeof(f32) * 3 + sizeof(f32) * 2
                        shader_location: 2,
                        format: VertexFormat::Uint32,
                    },
                ],
            },
            // Instance data (position and block_id)
            VertexBufferLayout {
                array_stride: std::mem::size_of::<InstanceData>() as u64,
                step_mode: VertexStepMode::Instance,
                attributes: &[
                    VertexAttribute {
                        offset: 0,
                        shader_location: 3,
                        format: VertexFormat::Float32x3,
                    },
                    VertexAttribute {
                        offset: 12,
                        shader_location: 4,
                        format: VertexFormat::Uint32,
                    },
                ],
            },
        ];

        // Create render pipeline layout
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create depth texture
        let depth_texture = device.create_texture(&TextureDescriptor {
            label: Some("Depth Texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&TextureViewDescriptor::default());

        // Create sampler for texture
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        // Create render pipeline
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &vertex_buffer_layouts,
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Rgba8UnormSrgb,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create staging buffer for GPU-to-CPU readback
        // Buffer size: width * height * 4 bytes (RGBA)
        let buffer_size = (width as u64 * height as u64 * 4) as u64;
        let staging_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Staging Buffer"),
            size: buffer_size,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        Self {
            device: Arc::new(device),
            queue: Arc::new(queue),
            render_pipeline,
            uniform_bind_group,
            uniform_buffer,
            texture_bind_group,
            texture,
            sampler,
            depth_texture,
            depth_view,
            staging_buffer,
            width,
            height,
        }
    }

    /// Load block textures and create a texture atlas
    fn load_texture_atlas(
        device: &Device,
        queue: &Queue,
    ) -> (
        Texture,
        BindGroup,
        std::collections::HashMap<(Block, u8), TextureUV>,
    ) {
        // Try to find resources directory relative to the executable
        let resource_paths = [
            Path::new("crafty/resources/block"),
            Path::new("../crafty/resources/block"),
            Path::new("../../crafty/resources/block"),
            Path::new("resources/block"),
        ];

        let mut block_dir = None;
        for path in &resource_paths {
            if path.exists() && path.is_dir() {
                block_dir = Some(path);
                break;
            }
        }

        let block_dir = block_dir.expect("Could not find resources/block directory");

        // Load all textures
        let texture_size: u32 = 64; // Assuming 64x64 textures
        let num_blocks = Block::iter().count();
        let textures_per_block = 3; // side, top, bottom
        let textures_per_row: u32 = 16; // Arrange in a grid
        let atlas_width = textures_per_row * texture_size;
        let atlas_height = ((num_blocks * textures_per_block + textures_per_row as usize - 1)
            / textures_per_row as usize) as u32
            * texture_size;

        let mut atlas = RgbaImage::new(atlas_width, atlas_height);
        let mut uv_map = std::collections::HashMap::new();

        let mut texture_idx = 0;
        for block in Block::iter() {
            let name = block.file_name();
            for (face_idx, face_name) in ["side", "top", "bottom"].iter().enumerate() {
                let texture_path = block_dir.join(format!("{}_{}.png", name, face_name));

                if let Ok(img) = image::open(&texture_path) {
                    let img = img.to_rgba8();
                    let row = texture_idx / textures_per_row as usize;
                    let col = texture_idx % textures_per_row as usize;
                    let x_offset = (col * texture_size as usize) as u32;
                    let y_offset = (row * texture_size as usize) as u32;

                    // Copy texture into atlas
                    for y in 0..texture_size {
                        for x in 0..texture_size {
                            let src_x = x.min(img.width() - 1);
                            let src_y = y.min(img.height() - 1);
                            let pixel = img.get_pixel(src_x, src_y);
                            atlas.put_pixel(x_offset + x, y_offset + y, *pixel);
                        }
                    }

                    // Calculate UV coordinates (normalized 0-1)
                    // Note: image coordinates have origin at top-left
                    let u_min = x_offset as f32 / atlas_width as f32;
                    let v_min = y_offset as f32 / atlas_height as f32;
                    let u_max = (x_offset + texture_size) as f32 / atlas_width as f32;
                    let v_max = (y_offset + texture_size) as f32 / atlas_height as f32;

                    uv_map.insert(
                        (block, face_idx as u8),
                        TextureUV {
                            u_min,
                            v_min,
                            u_max,
                            v_max,
                        },
                    );
                } else {
                    eprintln!("Warning: Could not load texture: {:?}", texture_path);
                }

                texture_idx += 1;
            }
        }

        // Create wgpu texture from atlas
        // Convert RgbaImage to raw bytes
        let atlas_bytes: Vec<u8> = atlas.into_raw();

        // Create texture
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Texture Atlas"),
            size: Extent3d {
                width: atlas_width,
                height: atlas_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Upload texture data
        queue.write_texture(
            ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &atlas_bytes,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some((atlas_width * 4) as u32),
                rows_per_image: Some(atlas_height),
            },
            Extent3d {
                width: atlas_width,
                height: atlas_height,
                depth_or_array_layers: 1,
            },
        );

        let texture_view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            view_dimension: TextureViewDimension::D2,
                            sample_type: TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        (texture, texture_bind_group, uv_map)
    }

    /// Render the scene from the player's perspective
    /// Returns raw RGB pixels as Vec<u8>
    pub fn render(&self, world: &World, player: &Player) -> Vec<u8> {
        // Get visible cubes - use cubes_near_player to get cubes
        let player_pos = player.position().pos();
        let mut visible_cubes = Vec::new();

        for cube_opt in world.cubes_near_player(player_pos) {
            if let Some(c) = cube_opt {
                if c.is_visible() {
                    visible_cubes.push(InstanceData {
                        position: c.position().as_array(),
                        block_id: c.block_id() as u32,
                    });
                }
            }
        }

        if visible_cubes.is_empty() {
            // Return sky blue gradient if no cubes
            return self.render_sky_gradient();
        }

        // Calculate matrices
        let perspective = perspective_matrix((self.width, self.height));
        let view = player.view_matrix();

        // Update uniform buffer
        let uniforms = Uniforms { perspective, view };
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        // Create vertex buffer for cube geometry
        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&VERTICES),
                usage: BufferUsages::VERTEX,
            });

        // Create instance buffer
        let instance_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&visible_cubes),
                usage: BufferUsages::VERTEX,
            });

        // Create output texture
        let output_texture = self.device.create_texture(&TextureDescriptor {
            label: Some("Output Texture"),
            size: Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let output_view = output_texture.create_view(&TextureViewDescriptor::default());

        // Create command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.53,
                            g: 0.81,
                            b: 0.92,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_bind_group(1, &self.texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
            render_pass.draw(0..VERTICES.len() as u32, 0..visible_cubes.len() as u32);
        }

        // Copy texture to staging buffer for readback
        encoder.copy_texture_to_buffer(
            ImageCopyTexture {
                texture: &output_texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            ImageCopyBuffer {
                buffer: &self.staging_buffer,
                layout: ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(self.width * 4),
                    rows_per_image: Some(self.height),
                },
            },
            Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );

        // Submit command buffer
        self.queue.submit(std::iter::once(encoder.finish()));

        // Wait for GPU to finish copying
        self.device.poll(wgpu::Maintain::Wait);

        // Map the staging buffer for reading
        let buffer_slice = self.staging_buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });

        // Poll until mapping is complete
        loop {
            self.device.poll(wgpu::Maintain::Wait);
            if let Ok(result) = receiver.try_recv() {
                result.expect("Buffer mapping failed");
                break;
            }
        }

        // Get the mapped data
        let mapped_range = buffer_slice.get_mapped_range();
        
        // Convert RGBA to RGB (remove alpha channel)
        let mut pixels = Vec::with_capacity((self.width * self.height * 3) as usize);
        for chunk in mapped_range.chunks_exact(4) {
            // RGBA format: [R, G, B, A]
            pixels.push(chunk[0]); // R
            pixels.push(chunk[1]); // G
            pixels.push(chunk[2]); // B
            // Skip alpha channel
        }
        
        // Unmap the buffer
        drop(mapped_range);
        self.staging_buffer.unmap();

        pixels
    }

    fn render_sky_gradient(&self) -> Vec<u8> {
        let mut pixels = Vec::with_capacity((self.width * self.height * 3) as usize);
        for y in 0..self.height {
            for _x in 0..self.width {
                let factor = 1.0 - (y as f32 / self.height as f32) * 0.3;
                pixels.push((0.53 * factor * 255.0) as u8);
                pixels.push((0.81 * factor * 255.0) as u8);
                pixels.push((0.92 * factor * 255.0) as u8);
            }
        }
        pixels
    }
}

use super::atlas::GlyphAtlas;
use super::DrawCommand;
use wgpu::util::DeviceExt;

const MAX_INSTANCES: usize = 16384;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct RectInstance {
    rect: [f32; 4],
    color: [f32; 4],
    border_color: [f32; 4],
    params: [f32; 2],
    _pad: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct TextGpuInstance {
    rect: [f32; 4],
    uv_rect: [f32; 4],
    color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    viewport: [f32; 2],
    _pad: [f32; 2],
}

const QUAD_VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.0],
    },
    Vertex {
        position: [1.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0],
    },
    Vertex {
        position: [0.0, 1.0],
    },
];

const QUAD_INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

/// GPU renderer that draws display-list commands via wgpu.
pub struct Renderer {
    rect_pipeline: wgpu::RenderPipeline,
    text_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    text_instance_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    atlas: GlyphAtlas,
    atlas_texture: wgpu::Texture,
    atlas_bind_group: wgpu::BindGroup,
}

impl Renderer {
    /// Create a new renderer for the given texture format.
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniform layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniforms"),
            contents: bytemuck::bytes_of(&Uniforms {
                viewport: [800.0, 600.0],
                _pad: [0.0; 2],
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniform bind group"),
            layout: &uniform_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // -- Rect pipeline --
        let rect_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rect shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let rect_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rect pipeline layout"),
            bind_group_layouts: &[&uniform_layout],
            push_constant_ranges: &[],
        });

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            }],
        };

        let rect_instance_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<RectInstance>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 16,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 32,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 48,
                    shader_location: 4,
                },
            ],
        };

        let blend_state = wgpu::BlendState::ALPHA_BLENDING;
        let color_target = wgpu::ColorTargetState {
            format,
            blend: Some(blend_state),
            write_mask: wgpu::ColorWrites::ALL,
        };

        let rect_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("rect pipeline"),
            layout: Some(&rect_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &rect_shader,
                entry_point: Some("vs_main"),
                buffers: &[vertex_layout.clone(), rect_instance_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &rect_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(color_target.clone())],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // -- Text pipeline --
        let text_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("text shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("text_shader.wgsl").into()),
        });

        let atlas = GlyphAtlas::new();

        let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("glyph atlas"),
            size: wgpu::Extent3d {
                width: atlas.size,
                height: atlas.size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let atlas_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let atlas_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("atlas sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let atlas_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("atlas layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("atlas bind group"),
            layout: &atlas_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&atlas_sampler),
                },
            ],
        });

        let text_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("text pipeline layout"),
            bind_group_layouts: &[&uniform_layout, &atlas_layout],
            push_constant_ranges: &[],
        });

        let text_instance_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TextGpuInstance>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 16,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 32,
                    shader_location: 3,
                },
            ],
        };

        let text_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("text pipeline"),
            layout: Some(&text_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &text_shader,
                entry_point: Some("vs_main"),
                buffers: &[vertex_layout, text_instance_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &text_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(color_target)],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // -- Shared buffers --
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad vertices"),
            contents: bytemuck::cast_slice(QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad indices"),
            contents: bytemuck::cast_slice(QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rect instances"),
            size: (MAX_INSTANCES * std::mem::size_of::<RectInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let text_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("text instances"),
            size: (MAX_INSTANCES * std::mem::size_of::<TextGpuInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            rect_pipeline,
            text_pipeline,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            text_instance_buffer,
            uniform_buffer,
            uniform_bind_group,
            atlas,
            atlas_texture,
            atlas_bind_group,
        }
    }

    /// Render draw commands to `target`. Viewport is in logical pixels.
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &wgpu::TextureView,
        viewport: [f32; 2],
        commands: &[DrawCommand],
    ) {
        let uniforms = Uniforms {
            viewport,
            _pad: [0.0; 2],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        let mut rect_instances: Vec<RectInstance> = Vec::new();
        let mut text_instances: Vec<TextGpuInstance> = Vec::new();

        for cmd in commands {
            match cmd {
                DrawCommand::Rect {
                    position,
                    size,
                    color,
                    corner_radius,
                    border_width,
                    border_color,
                } => {
                    rect_instances.push(RectInstance {
                        rect: [position[0], position[1], size[0], size[1]],
                        color: color.to_array(),
                        border_color: border_color.to_array(),
                        params: [*corner_radius, *border_width],
                        _pad: [0.0; 2],
                    });
                }
                DrawCommand::Text(batch) => {
                    for upload in &batch.uploads {
                        self.atlas.insert(
                            upload.cache_key,
                            upload.width,
                            upload.height,
                            &upload.data,
                        );
                    }
                    let atlas_size = self.atlas.size as f32;
                    for glyph in &batch.glyphs {
                        if let Some(region) = self.atlas.get(glyph.cache_key) {
                            text_instances.push(TextGpuInstance {
                                rect: [glyph.x, glyph.y, glyph.width as f32, glyph.height as f32],
                                uv_rect: [
                                    region.x as f32 / atlas_size,
                                    region.y as f32 / atlas_size,
                                    (region.x + region.width) as f32 / atlas_size,
                                    (region.y + region.height) as f32 / atlas_size,
                                ],
                                color: glyph.color.to_array(),
                            });
                        }
                    }
                }
            }
        }

        if self.atlas.dirty {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.atlas_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &self.atlas.data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.atlas.size * 4),
                    rows_per_image: Some(self.atlas.size),
                },
                wgpu::Extent3d {
                    width: self.atlas.size,
                    height: self.atlas.size,
                    depth_or_array_layers: 1,
                },
            );
            self.atlas.dirty = false;
        }

        let rect_count = rect_instances.len().min(MAX_INSTANCES);
        if !rect_instances.is_empty() {
            queue.write_buffer(
                &self.instance_buffer,
                0,
                bytemuck::cast_slice(&rect_instances[..rect_count]),
            );
        }

        let text_count = text_instances.len().min(MAX_INSTANCES);
        if !text_instances.is_empty() {
            queue.write_buffer(
                &self.text_instance_buffer,
                0,
                bytemuck::cast_slice(&text_instances[..text_count]),
            );
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("render encoder"),
        });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.08,
                            g: 0.08,
                            b: 0.10,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if rect_count > 0 {
                pass.set_pipeline(&self.rect_pipeline);
                pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
                pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                pass.draw_indexed(0..6, 0, 0..rect_count as u32);
            }

            if text_count > 0 {
                pass.set_pipeline(&self.text_pipeline);
                pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                pass.set_bind_group(1, &self.atlas_bind_group, &[]);
                pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                pass.set_vertex_buffer(1, self.text_instance_buffer.slice(..));
                pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                pass.draw_indexed(0..6, 0, 0..text_count as u32);
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}

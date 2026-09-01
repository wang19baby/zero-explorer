use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;

use ab_glyph::{FontVec, PxScale, GlyphId, Font, ScaleFont};

pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub shape_pipeline: Option<wgpu::RenderPipeline>,
    pub text_pipeline: Option<wgpu::RenderPipeline>,
    pub font: Option<FontData>,
    pub atlas: TextureAtlas,
}

pub struct FontData {
    pub font: FontVec,
    pub font_size: f32,
}

pub struct TextureAtlas {
    pub texture: Option<wgpu::Texture>,
    pub texture_view: Option<wgpu::TextureView>,
    pub bind_group: Option<wgpu::BindGroup>,
    pub size: u32,
    pub cursor_x: u32,
    pub cursor_y: u32,
    pub row_height: u32,
    pub glyph_cache: std::collections::HashMap<GlyphId, AtlasGlyph>,
}

#[derive(Clone)]
pub struct AtlasGlyph {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub advance: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ShapeVertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl ShapeVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ShapeVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TextVertex {
    position: [f32; 2],
    tex_coord: [f32; 2],
    color: [f32; 4],
}

impl TextVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TextVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

const SHAPE_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(in.position, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#;

const TEXT_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@group(0) @binding(0)
var t_texture: texture_2d<f32>;
@group(0) @binding(1)
var s_sampler: sampler;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(in.position, 0.0, 1.0);
    out.tex_coord = in.tex_coord;
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var tex_color = textureSample(t_texture, s_sampler, in.tex_coord);
    return vec4<f32>(in.color.rgb, in.color.a * tex_color.a);
}
"#;

impl TextureAtlas {
    fn new(device: &wgpu::Device, bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        let size = 1024;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Text Atlas"),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Text Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Text Bind Group"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            texture: Some(texture),
            texture_view: Some(texture_view),
            bind_group: Some(bind_group),
            size,
            cursor_x: 0,
            cursor_y: 0,
            row_height: 0,
            glyph_cache: std::collections::HashMap::new(),
        }
    }
}

impl GpuContext {
    pub fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window)?;

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .ok_or_else(|| anyhow::anyhow!("Failed to find suitable adapter"))?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Zero Explorer Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        ))?;

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create shape rendering pipeline
        let shape_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shape Shader"),
            source: wgpu::ShaderSource::Wgsl(SHAPE_SHADER.into()),
        });

        let shape_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Shape Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let shape_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Shape Pipeline"),
            layout: Some(&shape_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shape_shader,
                entry_point: "vs_main",
                buffers: &[ShapeVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shape_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create text rendering pipeline
        let text_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Text Shader"),
            source: wgpu::ShaderSource::Wgsl(TEXT_SHADER.into()),
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Text Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
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

        let text_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Text Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let text_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Text Pipeline"),
            layout: Some(&text_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &text_shader,
                entry_point: "vs_main",
                buffers: &[TextVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &text_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let atlas = TextureAtlas::new(&device, &texture_bind_group_layout);

        // Load font
        let font_data = Self::load_font();
        let font = FontVec::try_from_vec(font_data)
            .map_err(|e| anyhow::anyhow!("Failed to load font: {}", e))?;

        let font_size = 14.0;

        log::info!("GPU initialized: {:?}", adapter.get_info());

        Ok(Self {
            device,
            queue,
            surface,
            surface_config: config,
            shape_pipeline: Some(shape_pipeline),
            text_pipeline: Some(text_pipeline),
            font: Some(FontData { font, font_size }),
            atlas,
        })
    }

    fn load_font() -> Vec<u8> {
        let font_paths = [
            "C:\\Windows\\Fonts\\msyh.ttc",
            "C:\\Windows\\Fonts\\simhei.ttf",
            "C:\\Windows\\Fonts\\simsun.ttc",
            "C:\\Windows\\Fonts\\arial.ttf",
            "C:\\Windows\\Fonts\\segoeui.ttf",
        ];

        for path in &font_paths {
            if let Ok(data) = std::fs::read(path) {
                log::info!("Loaded font from: {}", path);
                return data;
            }
        }

        panic!("No font found!");
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    pub fn begin_frame(&mut self) -> Option<wgpu::SurfaceTexture> {
        match self.surface.get_current_texture() {
            Ok(frame) => Some(frame),
            Err(wgpu::SurfaceError::Lost) => {
                self.surface
                    .configure(&self.device, &self.surface_config);
                None
            }
            Err(e) => {
                log::error!("Surface error: {:?}", e);
                None
            }
        }
    }

    pub fn end_frame(&self, frame: wgpu::SurfaceTexture) {
        frame.present();
    }

    pub fn clear(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        color: wgpu::Color,
    ) {
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Clear Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        drop(render_pass);
    }

    pub fn draw_rect(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: [f32; 4],
        screen_width: f32,
        screen_height: f32,
    ) {
        let pipeline = match &self.shape_pipeline {
            Some(p) => p,
            None => return,
        };

        let ndc_x = (x / screen_width) * 2.0 - 1.0;
        let ndc_y = 1.0 - (y / screen_height) * 2.0;
        let ndc_w = (width / screen_width) * 2.0;
        let ndc_h = (height / screen_height) * 2.0;

        let vertices = vec![
            ShapeVertex { position: [ndc_x, ndc_y], color },
            ShapeVertex { position: [ndc_x + ndc_w, ndc_y], color },
            ShapeVertex { position: [ndc_x + ndc_w, ndc_y - ndc_h], color },
            ShapeVertex { position: [ndc_x, ndc_y - ndc_h], color },
        ];

        let indices: Vec<u16> = vec![0, 1, 2, 0, 2, 3];

        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Shape Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Shape Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Shape Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(pipeline);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..1);
    }

    pub fn draw_text(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        text: &str,
        x: f32,
        y: f32,
        color: [f32; 4],
        screen_width: f32,
        screen_height: f32,
    ) {
        let font_data = match &self.font {
            Some(f) => f,
            None => return,
        };

        let font = &font_data.font;
        let font_size = PxScale::from(font_data.font_size);
        let scale_font = font.as_scaled(font_size);

        let mut current_x = x;
        let mut vertices: Vec<TextVertex> = Vec::new();
        let mut indices: Vec<u16> = Vec::new();
        let mut vertex_count: u16 = 0;

        for char in text.chars() {
            let glyph_id = font.glyph_id(char);

            // Get or create glyph in atlas
            if !self.atlas.glyph_cache.contains_key(&glyph_id) {
                let glyph = glyph_id.with_scale_and_position(font_size, ab_glyph::point(0.0, 0.0));
                if let Some(outlined) = font.outline_glyph(glyph) {
                    let bounds = outlined.px_bounds();
                    let width = (bounds.width() as u32).max(1);
                    let height = (bounds.height() as u32).max(1);

                    // Check if we need to move to next row
                    if self.atlas.cursor_x + width > self.atlas.size {
                        self.atlas.cursor_x = 0;
                        self.atlas.cursor_y += self.atlas.row_height + 1;
                        self.atlas.row_height = 0;
                    }

                    // Check if we need new texture
                    if self.atlas.cursor_y + height > self.atlas.size {
                        // Skip rendering this glyph
                        current_x += scale_font.h_advance(glyph_id);
                        continue;
                    }

                    // Rasterize glyph - use RGBA format for better compatibility
                    let mut image = vec![0u8; (width * height * 4) as usize];
                    outlined.draw(|px, py, coverage| {
                        let idx = ((py * width + px) * 4) as usize;
                        if idx + 3 < image.len() {
                            image[idx] = 255;     // R
                            image[idx + 1] = 255; // G
                            image[idx + 2] = 255; // B
                            image[idx + 3] = (coverage * 255.0) as u8; // A
                        }
                    });

                    // Upload to texture
                    if let Some(texture) = &self.atlas.texture {
                        self.queue.write_texture(
                            wgpu::ImageCopyTexture {
                                texture,
                                mip_level: 0,
                                origin: wgpu::Origin3d {
                                    x: self.atlas.cursor_x,
                                    y: self.atlas.cursor_y,
                                    z: 0,
                                },
                                aspect: wgpu::TextureAspect::All,
                            },
                            &image,
                            wgpu::ImageDataLayout {
                                offset: 0,
                                bytes_per_row: Some(width * 4),
                                rows_per_image: Some(height),
                            },
                            wgpu::Extent3d {
                                width,
                                height,
                                depth_or_array_layers: 1,
                            },
                        );
                    }

                    // Cache glyph
                    let cache_entry = AtlasGlyph {
                        x: self.atlas.cursor_x,
                        y: self.atlas.cursor_y,
                        width,
                        height,
                        advance: scale_font.h_advance(glyph_id),
                    };

                    self.atlas.glyph_cache.insert(glyph_id, cache_entry);
                    self.atlas.cursor_x += width + 1;
                    self.atlas.row_height = self.atlas.row_height.max(height);
                }
            }

            if let Some(cached) = self.atlas.glyph_cache.get(&glyph_id) {
                let glyph_x = current_x;
                let glyph_y = y;

                // Calculate texture coordinates
                let tex_x1 = cached.x as f32 / self.atlas.size as f32;
                let tex_y1 = cached.y as f32 / self.atlas.size as f32;
                let tex_x2 = (cached.x + cached.width) as f32 / self.atlas.size as f32;
                let tex_y2 = (cached.y + cached.height) as f32 / self.atlas.size as f32;

                // Screen coordinates
                let screen_x1 = (glyph_x / screen_width) * 2.0 - 1.0;
                let screen_y1 = 1.0 - (glyph_y / screen_height) * 2.0;
                let screen_x2 = ((glyph_x + cached.width as f32) / screen_width) * 2.0 - 1.0;
                let screen_y2 = 1.0 - ((glyph_y + cached.height as f32) / screen_height) * 2.0;

                // Add vertices
                vertices.push(TextVertex {
                    position: [screen_x1, screen_y1],
                    tex_coord: [tex_x1, tex_y1],
                    color,
                });
                vertices.push(TextVertex {
                    position: [screen_x2, screen_y1],
                    tex_coord: [tex_x2, tex_y1],
                    color,
                });
                vertices.push(TextVertex {
                    position: [screen_x2, screen_y2],
                    tex_coord: [tex_x2, tex_y2],
                    color,
                });
                vertices.push(TextVertex {
                    position: [screen_x1, screen_y2],
                    tex_coord: [tex_x1, tex_y2],
                    color,
                });

                // Add indices
                indices.push(vertex_count);
                indices.push(vertex_count + 1);
                indices.push(vertex_count + 2);
                indices.push(vertex_count);
                indices.push(vertex_count + 2);
                indices.push(vertex_count + 3);
                vertex_count += 4;

                current_x += cached.advance;
            } else {
                current_x += scale_font.h_advance(glyph_id);
            }
        }

        // Draw text if we have vertices
        if !vertices.is_empty() {
            if let (Some(pipeline), Some(bind_group)) = (&self.text_pipeline, &self.atlas.bind_group) {
                let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Text Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

                let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Text Index Buffer"),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Text Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                render_pass.set_pipeline(pipeline);
                render_pass.set_bind_group(0, bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
            }
        }
    }

    pub fn measure_text(&self, text: &str) -> f32 {
        let font_data = match &self.font {
            Some(f) => f,
            None => return 0.0,
        };

        let font = &font_data.font;
        let font_size = PxScale::from(font_data.font_size);
        let scale_font = font.as_scaled(font_size);

        let mut width = 0.0;

        for char in text.chars() {
            let glyph_id = font.glyph_id(char);

            if let Some(cached) = self.atlas.glyph_cache.get(&glyph_id) {
                width += cached.advance;
            } else {
                width += scale_font.h_advance(glyph_id);
            }
        }

        width
    }
}

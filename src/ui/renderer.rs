use std::sync::Arc;
use std::num::NonZeroUsize;
use wgpu::util::DeviceExt;
use winit::window::Window;

use super::font_renderer::FontRenderer;
use super::font_loader::AsyncFontLoader;
use super::gpu_backend::{GpuBackend, GpuBackendInitializer};
use super::icons::FileIcon;
use super::shell_icons::{ShellIconExtractor, IconRequest, IconSize};
use super::theme_manager::{ThemeManager, ThemeType};

pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub shape_pipeline: Option<wgpu::RenderPipeline>,
    pub text_pipeline: Option<wgpu::RenderPipeline>,
    pub lcd_text_pipeline: Option<wgpu::RenderPipeline>,
    pub texture_pipeline: Option<wgpu::RenderPipeline>,
    pub font_renderer: Option<FontRenderer>,
    pub icon_font_renderer: Option<FontRenderer>,
    pub atlas: TextureAtlas,
    /// GPU后端信息
    pub backend: GpuBackend,
    /// 异步字体加载器
    pub font_loader: AsyncFontLoader,
    /// 主题管理器
    pub theme_manager: ThemeManager,
    /// Shell图标提取器
    pub icon_extractor: ShellIconExtractor,
    /// 图标请求ID计数器
    pub icon_request_id: u64,
}

pub struct RectParams<'a> {
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub view: &'a wgpu::TextureView,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub color: [f32; 4],
    pub screen_width: f32,
    pub screen_height: f32,
}

pub struct TextParams<'a> {
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub view: &'a wgpu::TextureView,
    pub text: &'a str,
    pub x: f32,
    pub y: f32,
    pub color: [f32; 4],
    pub screen_width: f32,
    pub screen_height: f32,
}

/// 纹理图集 - 管理字形和图标纹理
pub struct TextureAtlas {
    pub texture: Option<wgpu::Texture>,
    pub texture_view: Option<wgpu::TextureView>,
    pub bind_group: Option<wgpu::BindGroup>,
    pub size: u32,
    pub cursor_x: u32,
    pub cursor_y: u32,
    pub row_height: u32,
    pub glyph_positions: std::collections::HashMap<ab_glyph::GlyphId, AtlasPosition>,
    /// 图标纹理位置 (path -> AtlasPosition)
    pub icon_positions: std::collections::HashMap<String, AtlasPosition>,
}

#[derive(Clone, Debug)]
pub struct AtlasPosition {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
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

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TextureVertex {
    position: [f32; 2],
    tex_coord: [f32; 2],
}

impl TextureVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TextureVertex>() as wgpu::BufferAddress,
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

/// LCD 子像素抗锯齿着色器
/// 纹理存储 R/G/B 通道的 coverage，分别与文字颜色混合
const LCD_TEXT_SHADER: &str = r#"
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
    // LCD 子像素抗锯齿：R/G/B 通道分别作为 coverage
    // tex_color.r = R 子像素覆盖率
    // tex_color.g = G 子像素覆盖率
    // tex_color.b = B 子像素覆盖率
    var r = in.color.r * tex_color.r;
    var g = in.color.g * tex_color.g;
    var b = in.color.b * tex_color.b;
    // 使用 R 通道作为整体 alpha（假设 RGB 均匀覆盖）
    var a = in.color.a * (tex_color.r + tex_color.g + tex_color.b) / 3.0;
    return vec4<f32>(r, g, b, a);
}
"#;

const TEXTURE_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coord: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
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
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_texture, s_sampler, in.tex_coord);
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
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
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
            glyph_positions: std::collections::HashMap::new(),
            icon_positions: std::collections::HashMap::new(),
        }
    }
    
    /// 上传字形到纹理图集
    pub fn upload_glyph(
        &mut self,
        glyph_id: ab_glyph::GlyphId,
        width: u32,
        height: u32,
        pixels: &[u8],
        queue: &wgpu::Queue,
    ) -> Option<AtlasPosition> {
        // 检查是否已经存在
        if let Some(pos) = self.glyph_positions.get(&glyph_id) {
            return Some(pos.clone());
        }
        
        // 检查是否有足够空间
        if self.cursor_x + width > self.size {
            // 换行
            self.cursor_x = 0;
            self.cursor_y += self.row_height + 1;
            self.row_height = 0;
        }
        
        if self.cursor_y + height > self.size {
            // 纹理已满
            log::warn!("Texture atlas full, cannot upload glyph");
            return None;
        }
        
        // 上传像素数据
        if let Some(texture) = &self.texture {
            log::debug!("upload_glyph id={} size={}x{} at ({},{}) pixels_len={}", 
                glyph_id.0, width, height, self.cursor_x, self.cursor_y, pixels.len());
            
            // 跳过空像素数据（如空格字符）
            if !pixels.is_empty() {
                queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: self.cursor_x,
                            y: self.cursor_y,
                            z: 0,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    pixels,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(width * 4), // RGBA = 4 bytes per pixel
                        rows_per_image: Some(height),
                    },
                    wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                );
            }
        }
        
        let pos = AtlasPosition {
            x: self.cursor_x,
            y: self.cursor_y,
            width,
            height,
        };
        
        self.glyph_positions.insert(glyph_id, pos.clone());
        self.cursor_x += width + 1;
        self.row_height = self.row_height.max(height);
        
        Some(pos)
    }

    /// 上传图标像素到纹理图集
    pub fn upload_icon(
        &mut self,
        path: &str,
        width: u32,
        height: u32,
        pixels: &[u8],
        queue: &wgpu::Queue,
    ) -> Option<AtlasPosition> {
        // 检查是否已经存在
        if let Some(pos) = self.icon_positions.get(path) {
            return Some(pos.clone());
        }
        
        // 检查是否有足够空间
        if self.cursor_x + width > self.size {
            // 换行
            self.cursor_x = 0;
            self.cursor_y += self.row_height + 1;
            self.row_height = 0;
        }
        
        if self.cursor_y + height > self.size {
            // 纹理已满
            log::warn!("Texture atlas full, cannot upload icon");
            return None;
        }
        
        // 上传像素数据
        if let Some(texture) = &self.texture {
            log::debug!("upload_icon path={} size={}x{} at ({},{})", 
                path, width, height, self.cursor_x, self.cursor_y);
            
            // 跳过空像素数据
            if !pixels.is_empty() {
                queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: self.cursor_x,
                            y: self.cursor_y,
                            z: 0,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    pixels,
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
        }
        
        let pos = AtlasPosition {
            x: self.cursor_x,
            y: self.cursor_y,
            width,
            height,
        };
        
        self.icon_positions.insert(path.to_string(), pos.clone());
        self.cursor_x += width + 1;
        self.row_height = self.row_height.max(height);
        
        Some(pos)
    }
}

impl GpuContext {
    pub fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();
        log::info!("GpuContext::new: window size={}x{}", size.width, size.height);

        // 检测可用后端
        let backends = GpuBackendInitializer::detect_available_backends();
        log::info!("Available backends: {:?}", backends.iter().map(|b| b.name()).collect::<Vec<_>>());
        let backend = backends.first().copied().unwrap_or(GpuBackend::Dx12);
        log::info!("Selected backend: {}", backend.name());

        // 使用检测到的后端创建Instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: match backend {
                GpuBackend::Dx12 => wgpu::Backends::DX12,
                GpuBackend::Vulkan => wgpu::Backends::VULKAN,
                GpuBackend::Gl => wgpu::Backends::GL,
            },
            ..Default::default()
        });

        // 创建Surface
        let surface = instance.create_surface(window)?;

        // 请求适配器
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .ok_or_else(|| anyhow::anyhow!("Failed to find GPU adapter"))?;

        let adapter_info = adapter.get_info();
        log::info!("GPU adapter: {} ({})", adapter_info.name, adapter_info.backend as u32);

        // 请求设备
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Zero Explorer GPU"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits {
                    max_texture_dimension_2d: 8192,
                    ..Default::default()
                },
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

        // 创建形状渲染管线
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

        // 创建文本渲染管线
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

        // 创建 LCD 文本渲染管线
        let lcd_text_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("LCD Text Shader"),
            source: wgpu::ShaderSource::Wgsl(LCD_TEXT_SHADER.into()),
        });

        let lcd_text_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("LCD Text Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let lcd_text_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("LCD Text Pipeline"),
            layout: Some(&lcd_text_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &lcd_text_shader,
                entry_point: "vs_main",
                buffers: &[TextVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &lcd_text_shader,
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

        // 创建纹理渲染管线 (用于图标)
        let texture_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Texture Shader"),
            source: wgpu::ShaderSource::Wgsl(TEXTURE_SHADER.into()),
        });

        let texture_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Texture Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let texture_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Texture Pipeline"),
            layout: Some(&texture_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &texture_shader,
                entry_point: "vs_main",
                buffers: &[TextureVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &texture_shader,
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

        // 启动异步字体加载
        let font_loader = AsyncFontLoader::new();
        font_loader.start_loading();

        // 等待字体加载完成 (阻塞)
        let font_data = font_loader.wait();

        // 克隆 primary 和 icon 给图标渲染器用
        let icon_primary_data = font_data.primary.clone();
        let icon_nerd_data = font_data.icon.clone();

        // 使用加载的字体初始化渲染器
        let mut font_renderer = FontRenderer::new(font_data.primary, 16.0)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        
        // 加载 CJK 回退字体
        if let Some(cjk_data) = font_data.cjk_sc {
            if let Err(e) = font_renderer.add_fallback_font(cjk_data) {
                log::warn!("Failed to load CJK font: {}", e);
            }
        }

        // 加载 Nerd Font（用于文件类型图标）
        if let Some(icon_data) = font_data.icon {
            if let Err(e) = font_renderer.add_fallback_font(icon_data) {
                log::warn!("Failed to load Nerd Font: {}", e);
            } else {
                log::info!("Loaded Nerd Font for file icons");
            }
        }

        // 创建图标专用字体渲染器（更大字号）
        log::info!("Creating icon font renderer");
        let mut icon_font_renderer = FontRenderer::new(icon_primary_data, 20.0)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        log::info!("Icon font renderer created");

        // 给图标渲染器也加载 Nerd Font
        if let Some(icon_data) = icon_nerd_data {
            if let Err(e) = icon_font_renderer.add_fallback_font(icon_data) {
                log::warn!("Failed to load Nerd Font for icon renderer: {}", e);
            } else {
                log::info!("Loaded Nerd Font for icon renderer");
            }
        }

        // 创建主题管理器 (默认深色主题)
        log::info!("Creating theme manager");
        let theme_manager = ThemeManager::new(ThemeType::Dark);

        // 创建Shell图标提取器
        log::info!("Creating shell icon extractor");
        let icon_extractor = ShellIconExtractor::new(NonZeroUsize::new(1000).unwrap());

        log::info!("GPU initialized with backend: {}", backend.name());
        log::info!("Font renderer initialized with {} cached glyphs", 
            font_renderer.glyph_cache().len());

        Ok(Self {
            device,
            queue,
            surface,
            surface_config: config,
            shape_pipeline: Some(shape_pipeline),
            text_pipeline: Some(text_pipeline),
            lcd_text_pipeline: Some(lcd_text_pipeline),
            texture_pipeline: Some(texture_pipeline),
            font_renderer: Some(font_renderer),
            icon_font_renderer: Some(icon_font_renderer),
            atlas,
            backend,
            font_loader,
            theme_manager,
            icon_extractor,
            icon_request_id: 0,
        })
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

    pub fn draw_rect(&self, params: RectParams) {
        let pipeline = match &self.shape_pipeline {
            Some(p) => p,
            None => return,
        };

        let ndc_x = (params.x / params.screen_width) * 2.0 - 1.0;
        let ndc_y = 1.0 - (params.y / params.screen_height) * 2.0;
        let ndc_w = (params.width / params.screen_width) * 2.0;
        let ndc_h = (params.height / params.screen_height) * 2.0;

        let vertices = vec![
            ShapeVertex { position: [ndc_x, ndc_y], color: params.color },
            ShapeVertex { position: [ndc_x + ndc_w, ndc_y], color: params.color },
            ShapeVertex { position: [ndc_x + ndc_w, ndc_y - ndc_h], color: params.color },
            ShapeVertex { position: [ndc_x, ndc_y - ndc_h], color: params.color },
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

        let mut render_pass = params.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Shape Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: params.view,
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

    pub fn draw_text(&mut self, params: TextParams) {
        // 选择渲染管线：LCD 或标准
        let use_lcd = self.font_renderer.as_ref()
            .map(|fr| fr.settings.lcd_subpixel_aa)
            .unwrap_or(false);
        
        let pipeline = if use_lcd {
            match &self.lcd_text_pipeline {
                Some(p) => p,
                None => match &self.text_pipeline {
                    Some(p) => p,
                    None => return,
                },
            }
        } else {
            match &self.text_pipeline {
                Some(p) => p,
                None => return,
            }
        };

        let mut current_x = params.x;
        let mut vertices: Vec<TextVertex> = Vec::new();
        let mut indices: Vec<u16> = Vec::new();
        let mut vertex_count: u16 = 0;

        for ch in params.text.chars() {
            // 计算子像素偏移
            let subpixel_offset = current_x - current_x.floor();
            
            // 获取字形（带子像素定位）
            let glyph_data = if let Some(fr) = &mut self.font_renderer {
                fr.get_glyph(ch, subpixel_offset).cloned()
            } else {
                None
            };
            
            let glyph_id = self.font_renderer.as_ref()
                .map(|fr| fr.glyph_id(ch))
                .unwrap_or_default();
            
            if let Some(glyph_data) = glyph_data {
                // 确保字形在纹理图集中
                let atlas_pos = if let Some(pos) = self.atlas.glyph_positions.get(&glyph_id) {
                    pos.clone()
                } else {
                    // 上传字形到纹理（优先使用 LCD 数据）
                    let pixels = glyph_data.lcd_pixels.as_ref()
                        .unwrap_or(&glyph_data.pixels);
                    match self.atlas.upload_glyph(
                        glyph_id,
                        glyph_data.width,
                        glyph_data.height,
                        pixels,
                        &self.queue,
                    ) {
                        Some(pos) => pos,
                        None => {
                            current_x += glyph_data.advance;
                            continue;
                        }
                    }
                };

                // 应用子像素偏移到渲染位置
                let glyph_x = current_x + glyph_data.bearing_x;
                let glyph_y = params.y + glyph_data.bearing_y;
                
                let tex_x1 = atlas_pos.x as f32 / self.atlas.size as f32;
                let tex_y1 = atlas_pos.y as f32 / self.atlas.size as f32;
                let tex_x2 = (atlas_pos.x + atlas_pos.width) as f32 / self.atlas.size as f32;
                let tex_y2 = (atlas_pos.y + atlas_pos.height) as f32 / self.atlas.size as f32;
                
                let screen_x1 = (glyph_x / params.screen_width) * 2.0 - 1.0;
                let screen_y1 = 1.0 - (glyph_y / params.screen_height) * 2.0;
                let screen_x2 = ((glyph_x + atlas_pos.width as f32) / params.screen_width) * 2.0 - 1.0;
                let screen_y2 = 1.0 - ((glyph_y + atlas_pos.height as f32) / params.screen_height) * 2.0;
                
                vertices.push(TextVertex {
                    position: [screen_x1, screen_y1],
                    tex_coord: [tex_x1, tex_y1],
                    color: params.color,
                });
                vertices.push(TextVertex {
                    position: [screen_x2, screen_y1],
                    tex_coord: [tex_x2, tex_y1],
                    color: params.color,
                });
                vertices.push(TextVertex {
                    position: [screen_x2, screen_y2],
                    tex_coord: [tex_x2, tex_y2],
                    color: params.color,
                });
                vertices.push(TextVertex {
                    position: [screen_x1, screen_y2],
                    tex_coord: [tex_x1, tex_y2],
                    color: params.color,
                });
                
                indices.push(vertex_count);
                indices.push(vertex_count + 1);
                indices.push(vertex_count + 2);
                indices.push(vertex_count);
                indices.push(vertex_count + 2);
                indices.push(vertex_count + 3);
                vertex_count += 4;
                
                current_x += glyph_data.advance;
            } else {
                let font_size = self.font_renderer.as_ref().map(|fr| fr.font_size()).unwrap_or(16.0);
                current_x += font_size * 0.5;
            }
        }

        // 渲染文字
        if !vertices.is_empty() {
            log::debug!("draw_text '{}': {} vertices, {} indices", params.text, vertices.len(), indices.len());
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

            let mut render_pass = params.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Text Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: params.view,
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
            if let Some(bind_group) = &self.atlas.bind_group {
                render_pass.set_bind_group(0, bind_group, &[]);
            }
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
        }
    }

    pub fn measure_text(&self, text: &str) -> f32 {
        self.font_renderer
            .as_ref()
            .map(|fr| fr.measure_text(text))
            .unwrap_or(0.0)
    }

    pub fn line_height(&self) -> f32 {
        self.font_renderer
            .as_ref()
            .map(|fr| fr.line_height())
            .unwrap_or(16.0)
    }

    pub fn ascent(&self) -> f32 {
        self.font_renderer
            .as_ref()
            .map(|fr| fr.ascent())
            .unwrap_or(12.0)
    }

    /// 获取当前主题颜色
    pub fn theme_colors(&self) -> &super::theme_manager::ThemeColors {
        self.theme_manager.colors()
    }

    /// 切换主题
    pub fn toggle_theme(&mut self) {
        self.theme_manager.toggle();
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_rect_simple(
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
        self.draw_rect(RectParams {
            encoder,
            view,
            x,
            y,
            width,
            height,
            color,
            screen_width,
            screen_height,
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_text_simple(
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
        self.draw_text(TextParams {
            encoder,
            view,
            text,
            x,
            y,
            color,
            screen_width,
            screen_height,
        });
    }

    /// 绘制纹理四边形 (用于图标渲染)
    #[allow(clippy::too_many_arguments)]
    pub fn draw_texture(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        atlas_pos: &AtlasPosition,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        screen_width: f32,
        screen_height: f32,
    ) {
        let pipeline = match &self.texture_pipeline {
            Some(p) => p,
            None => return,
        };

        let ndc_x = (x / screen_width) * 2.0 - 1.0;
        let ndc_y = 1.0 - (y / screen_height) * 2.0;
        let ndc_w = (width / screen_width) * 2.0;
        let ndc_h = (height / screen_height) * 2.0;

        let tex_x1 = atlas_pos.x as f32 / self.atlas.size as f32;
        let tex_y1 = atlas_pos.y as f32 / self.atlas.size as f32;
        let tex_x2 = (atlas_pos.x + atlas_pos.width) as f32 / self.atlas.size as f32;
        let tex_y2 = (atlas_pos.y + atlas_pos.height) as f32 / self.atlas.size as f32;

        let vertices = vec![
            TextureVertex { position: [ndc_x, ndc_y], tex_coord: [tex_x1, tex_y1] },
            TextureVertex { position: [ndc_x + ndc_w, ndc_y], tex_coord: [tex_x2, tex_y1] },
            TextureVertex { position: [ndc_x + ndc_w, ndc_y - ndc_h], tex_coord: [tex_x2, tex_y2] },
            TextureVertex { position: [ndc_x, ndc_y - ndc_h], tex_coord: [tex_x1, tex_y2] },
        ];

        let indices: Vec<u16> = vec![0, 1, 2, 0, 2, 3];

        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Texture Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Texture Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Texture Render Pass"),
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
        if let Some(bind_group) = &self.atlas.bind_group {
            render_pass.set_bind_group(0, bind_group, &[]);
        }
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..1);
    }

    /// 请求Shell图标 (异步)
    pub fn request_shell_icon(&mut self, path: &str, size: IconSize) -> u64 {
        self.icon_request_id += 1;
        let request = IconRequest {
            id: self.icon_request_id,
            path: path.to_string(),
            size,
        };
        self.icon_extractor.request(request);
        self.icon_request_id
    }

    /// 处理图标响应并上传到纹理图集
    pub fn process_icon_responses(&mut self) {
        while let Some(response) = self.icon_extractor.try_receive() {
            if let Some(pixels) = response.pixels {
                self.atlas.upload_icon(
                    &response.path,
                    response.width,
                    response.height,
                    &pixels,
                    &self.queue,
                );
            }
        }
    }

    /// 绘制文件图标：优先使用Shell图标纹理，回退到Nerd Font
    #[allow(clippy::too_many_arguments)]
    pub fn draw_file_icon(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        icon: FileIcon,
        icon_color: [f32; 4],
        path: &str,
        x: f32,
        y: f32,
        icon_size: f32,
        screen_width: f32,
        screen_height: f32,
    ) {
        // 尝试从纹理图集获取Shell图标
        if let Some(atlas_pos) = self.atlas.icon_positions.get(path).cloned() {
            // 使用纹理渲染
            self.draw_texture(
                encoder,
                view,
                &atlas_pos,
                x,
                y,
                icon_size,
                icon_size,
                screen_width,
                screen_height,
            );
        } else {
            // 回退到Nerd Font字符渲染
            let icon_str = icon.icon_char_str();
            let text_x = x + icon_size * 0.15;
            let text_y = y + icon_size * 0.85;
            
            let saved_font_renderer = self.font_renderer.take();
            self.font_renderer = self.icon_font_renderer.take();
            
            self.draw_text_simple(encoder, view, &icon_str, text_x, text_y, icon_color, screen_width, screen_height);
            
            self.icon_font_renderer = self.font_renderer.take();
            self.font_renderer = saved_font_renderer;
        }
    }
}

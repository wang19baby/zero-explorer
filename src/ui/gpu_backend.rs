use std::sync::Arc;
use winit::window::Window;

/// GPU后端类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuBackend {
    Dx12,
    Vulkan,
    Gl,
}

impl GpuBackend {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Dx12 => "DirectX 12",
            Self::Vulkan => "Vulkan",
            Self::Gl => "OpenGL",
        }
    }
}

/// GPU适配器信息
#[derive(Debug)]
pub struct AdapterInfo {
    pub name: String,
    pub backend: GpuBackend,
    pub device_type: wgpu::DeviceType,
    pub vendor: u32,
}

/// 多后端GPU初始化器
/// 参考 MTT File Manager 的三级降级策略: DX12 → Vulkan → OpenGL
pub struct GpuBackendInitializer {
    backends: Vec<GpuBackend>,
}

impl GpuBackendInitializer {
    /// 创建初始化器，自动检测可用后端
    pub fn new() -> Self {
        let backends = Self::detect_available_backends();
        log::trace!("Available GPU backends: {:?}", backends);
        Self { backends }
    }

    /// 检测当前平台可用的后端
    pub fn detect_available_backends() -> Vec<GpuBackend> {
        let mut backends = Vec::new();

        #[cfg(target_os = "windows")]
        {
            // Windows: DX12优先，Vulkan次之，OpenGL最后
            backends.push(GpuBackend::Dx12);
            backends.push(GpuBackend::Vulkan);
            backends.push(GpuBackend::Gl);
        }

        #[cfg(target_os = "linux")]
        {
            // Linux: Vulkan优先，OpenGL次之
            backends.push(GpuBackend::Vulkan);
            backends.push(GpuBackend::Gl);
        }

        #[cfg(target_os = "macos")]
        {
            // macOS: Metal (通过wgpu后端)
            // wgpu在macOS上使用Metal，不需要额外处理
            backends.push(GpuBackend::Vulkan); // Metal via wgpu
        }

        backends
    }

    /// 将GpuBackend转换为wgpu::Backends
    fn to_wgpu_backends(backend: GpuBackend) -> wgpu::Backends {
        match backend {
            GpuBackend::Dx12 => wgpu::Backends::DX12,
            GpuBackend::Vulkan => wgpu::Backends::VULKAN,
            GpuBackend::Gl => wgpu::Backends::GL,
        }
    }

    /// 按优先级尝试初始化GPU
    /// 返回 (device, queue, adapter_info, backend_used)
    pub fn try_initialize(
        &self,
        window: Arc<Window>,
    ) -> Result<(wgpu::Device, wgpu::Queue, AdapterInfo, GpuBackend), anyhow::Error> {
        let mut last_error = None;

        for &backend in &self.backends {
            log::trace!("Trying GPU backend: {}", backend.name());

            match self.try_init_backend(backend, window.clone()) {
                Ok((device, queue, info)) => {
                    log::trace!(
                        "GPU initialized successfully with {}: {}",
                        backend.name(),
                        info.name
                    );
                    return Ok((device, queue, info, backend));
                }
                Err(e) => {
                    log::warn!("Failed to initialize {}: {}", backend.name(), e);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("No GPU backends available")))
    }

    /// 尝试使用指定后端初始化
    fn try_init_backend(
        &self,
        backend: GpuBackend,
        window: Arc<Window>,
    ) -> Result<(wgpu::Device, wgpu::Queue, AdapterInfo), anyhow::Error> {
        let wgpu_backends = Self::to_wgpu_backends(backend);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu_backends,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone())?;

        // 请求适配器
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .ok_or_else(|| anyhow::anyhow!("Failed to find adapter for {}", backend.name()))?;

        let info = adapter.get_info();
        let adapter_info = AdapterInfo {
            name: info.name.clone(),
            backend,
            device_type: info.device_type,
            vendor: info.vendor,
        };

        // 请求设备
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some(&format!("Zero Explorer ({})", backend.name())),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits {
                    max_texture_dimension_2d: 8192,
                    ..Default::default()
                },
            },
            None,
        ))?;

        // 配置Surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let size = window.inner_size();
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

        Ok((device, queue, adapter_info))
    }
}

impl Default for GpuBackendInitializer {
    fn default() -> Self {
        Self::new()
    }
}

/// 检测系统主题 (Windows)
pub fn detect_system_theme() -> bool {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("reg")
            .args([
                "query",
                r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Themes\Personalize",
                "/v",
                "AppsUseLightTheme",
                "/t",
                "REG_DWORD",
            ])
            .output()
            .map(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.contains("0x1") // 0x1 = 浅色, 0x0 = 深色
            })
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false // 默认深色
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_available_backends() {
        let backends = GpuBackendInitializer::detect_available_backends();
        assert!(!backends.is_empty());
    }

    #[test]
    fn test_backend_name() {
        assert_eq!(GpuBackend::Dx12.name(), "DirectX 12");
        assert_eq!(GpuBackend::Vulkan.name(), "Vulkan");
        assert_eq!(GpuBackend::Gl.name(), "OpenGL");
    }
}

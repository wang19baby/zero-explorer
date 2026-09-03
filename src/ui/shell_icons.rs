use std::sync::{Arc, Mutex};
use crossbeam_channel::{bounded, Receiver, Sender};
use lru::LruCache;
use std::num::NonZeroUsize;

/// 图标请求
#[derive(Debug, Clone)]
pub struct IconRequest {
    pub id: u64,
    pub path: String,
    pub size: IconSize,
}

/// 图标响应
#[derive(Debug, Clone)]
pub struct IconResponse {
    pub id: u64,
    pub path: String,
    pub size: IconSize,
    pub pixels: Option<Vec<u8>>, // RGBA像素数据
    pub width: u32,
    pub height: u32,
}

/// 图标尺寸
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IconSize {
    Small,   // 16x16
    Medium,  // 32x32
    Large,   // 48x48
    ExtraLarge, // 64x64
}

impl IconSize {
    pub fn pixels(&self) -> u32 {
        match self {
            Self::Small => 16,
            Self::Medium => 32,
            Self::Large => 48,
            Self::ExtraLarge => 64,
        }
    }
}

/// Shell图标提取器
/// 参考 MTT File Manager 的图标加载架构: 4线程 + LRU缓存
pub struct ShellIconExtractor {
    /// 请求发送通道
    request_tx: Sender<IconRequest>,
    /// 响应发送通道
    response_tx: Sender<IconResponse>,
    /// 响应接收通道
    response_rx: Receiver<IconResponse>,
    /// LRU缓存
    cache: Arc<Mutex<LruCache<String, Vec<u8>>>>,
    /// 待处理请求计数
    pending_count: Arc<Mutex<u32>>,
}

impl ShellIconExtractor {
    /// 创建新的图标提取器
    pub fn new(cache_size: NonZeroUsize) -> Self {
        let (request_tx, request_rx) = bounded(1024);
        let (response_tx, response_rx) = bounded(1024);
        let cache = Arc::new(Mutex::new(LruCache::new(cache_size)));
        let pending_count = Arc::new(Mutex::new(0));

        // 启动4个工作线程
        for i in 0..4 {
            let rx = request_rx.clone();
            let tx = response_tx.clone();
            let cache = Arc::clone(&cache);
            let pending = Arc::clone(&pending_count);

            std::thread::spawn(move || {
                Self::worker_loop(i, rx, tx, cache, pending);
            });
        }

        Self {
            request_tx,
            response_tx,
            response_rx,
            cache,
            pending_count,
        }
    }

    /// 请求图标
    pub fn request(&self, request: IconRequest) {
        // 先检查缓存
        let cache_key = format!("{}:{}", request.path, request.size.pixels());
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(pixels) = cache.get(&cache_key) {
                // 缓存命中，直接返回
                let _ = self.response_tx.send(IconResponse {
                    id: request.id,
                    path: request.path,
                    size: request.size,
                    pixels: Some(pixels.clone()),
                    width: request.size.pixels(),
                    height: request.size.pixels(),
                });
                return;
            }
        }

        // 缓存未命中，发送到工作线程
        {
            let mut pending = self.pending_count.lock().unwrap();
            *pending += 1;
        }
        let _ = self.request_tx.send(request);
    }

    /// 尝试获取响应 (非阻塞)
    pub fn try_receive(&self) -> Option<IconResponse> {
        self.response_rx.try_recv().ok()
    }

    /// 获取待处理请求数
    pub fn pending_count(&self) -> u32 {
        *self.pending_count.lock().unwrap()
    }

    /// 清空缓存
    pub fn clear_cache(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }

    /// 工作线程主循环
    fn worker_loop(
        id: usize,
        rx: Receiver<IconRequest>,
        tx: Sender<IconResponse>,
        cache: Arc<Mutex<LruCache<String, Vec<u8>>>>,
        pending: Arc<Mutex<u32>>,
    ) {
        log::trace!("Icon worker {} started", id);

        while let Ok(request) = rx.recv() {
            let result = Self::extract_icon(&request.path, request.size);

            // 更新缓存
            if let Some(ref pixels) = result {
                let cache_key = format!("{}:{}", request.path, request.size.pixels());
                let mut cache = cache.lock().unwrap();
                cache.put(cache_key, pixels.clone());
            }

            // 发送响应
            let _ = tx.send(IconResponse {
                id: request.id,
                path: request.path,
                size: request.size,
                pixels: result,
                width: request.size.pixels(),
                height: request.size.pixels(),
            });

            // 减少待处理计数
            {
                let mut pending = pending.lock().unwrap();
                *pending = pending.saturating_sub(1);
            }
        }

        log::trace!("Icon worker {} stopped", id);
    }

    /// 提取文件图标
    fn extract_icon(path: &str, size: IconSize) -> Option<Vec<u8>> {
        #[cfg(target_os = "windows")]
        {
            Self::extract_icon_windows(path, size)
        }
        #[cfg(not(target_os = "windows"))]
        {
            Self::extract_icon_fallback(path, size)
        }
    }

    /// Windows图标提取 - 使用SHGetFileInfo + GetIconInfo + GetDIBits
    #[cfg(target_os = "windows")]
    fn extract_icon_windows(path: &str, size: IconSize) -> Option<Vec<u8>> {
        use windows::core::PCWSTR;
        use windows::Win32::Graphics::Gdi::{
            DeleteObject, GetDC, ReleaseDC,
            CreateCompatibleDC, SelectObject, GetDIBits,
            BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS,
        };
        use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
        use windows::Win32::UI::Shell::{
            SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON,
        };
        use windows::Win32::UI::WindowsAndMessaging::{
            DestroyIcon, GetIconInfo, ICONINFO,
        };

        let path_wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
        let icon_size = size.pixels() as i32;

        unsafe {
            // 1. 使用SHGetFileInfo获取HICON
            let mut file_info = SHFILEINFOW::default();
            let result = SHGetFileInfoW(
                PCWSTR(path_wide.as_ptr()),
                FILE_FLAGS_AND_ATTRIBUTES(0),
                Some(&mut file_info),
                0,
                SHGFI_ICON,
            );

            if result == 0 || file_info.hIcon.is_invalid() {
                log::warn!("SHGetFileInfo failed for: {}", path);
                return None;
            }

            let hicon = file_info.hIcon;

            // 2. 获取ICONINFO
            let mut icon_info = ICONINFO::default();
            if GetIconInfo(hicon, &mut icon_info).is_err() {
                log::warn!("GetIconInfo failed for: {}", path);
                let _ = DestroyIcon(hicon);
                return None;
            }

            let hbm_color = icon_info.hbmColor;
            let hbm_mask = icon_info.hbmMask;

            // 3. 获取DC
            let screen_dc = GetDC(None);
            let mem_dc = CreateCompatibleDC(screen_dc);

            // 4. 准备BITMAPINFO
            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: icon_size,
                    biHeight: -icon_size, // 自上而下
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: 0, // BI_RGB
                    ..Default::default()
                },
                ..Default::default()
            };

            // 5. 提取颜色位图
            let mut color_pixels = vec![0u8; (icon_size * icon_size * 4) as usize];
            let old_bmp = SelectObject(mem_dc, hbm_color);
            GetDIBits(
                mem_dc,
                hbm_color,
                0,
                icon_size as u32,
                Some(color_pixels.as_mut_ptr() as *mut _),
                &mut bmi,
                DIB_RGB_COLORS,
            );
            SelectObject(mem_dc, old_bmp);

            // 6. 提取掩码位图 (用于Alpha通道)
            let mut mask_pixels = vec![0u8; (icon_size * icon_size * 4) as usize];
            let old_bmp = SelectObject(mem_dc, hbm_mask);
            GetDIBits(
                mem_dc,
                hbm_mask,
                0,
                icon_size as u32,
                Some(mask_pixels.as_mut_ptr() as *mut _),
                &mut bmi,
                DIB_RGB_COLORS,
            );
            SelectObject(mem_dc, old_bmp);

            // 7. 合并Alpha通道
            // Windows ICO格式: 颜色位图是BGRA，掩码位图的蓝色通道是Alpha
            let mut rgba_pixels = vec![0u8; (icon_size * icon_size * 4) as usize];
            for y in 0..icon_size as usize {
                for x in 0..icon_size as usize {
                    let idx = (y * icon_size as usize + x) * 4;
                    let src_idx = idx;

                    // BGRA → RGBA
                    rgba_pixels[idx] = color_pixels[src_idx + 2];     // R
                    rgba_pixels[idx + 1] = color_pixels[src_idx + 1]; // G
                    rgba_pixels[idx + 2] = color_pixels[src_idx];     // B

                    // Alpha: 从掩码位图获取 (如果位图为0则不透明)
                    let mask_bit = mask_pixels[src_idx] & 0x01;
                    rgba_pixels[idx + 3] = if mask_bit == 0 { 255 } else { 0 };
                }
            }

            // 8. 清理资源
            let _ = DeleteObject(hbm_color);
            let _ = DeleteObject(hbm_mask);
            ReleaseDC(None, screen_dc);
            let _ = DestroyIcon(hicon);

            Some(rgba_pixels)
        }
    }

    /// 非Windows平台的回退实现
    #[cfg(not(target_os = "windows"))]
    fn extract_icon_fallback(path: &str, size: IconSize) -> Option<Vec<u8>> {
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let color = match ext.to_lowercase().as_str() {
            "exe" | "msi" | "bat" | "cmd" | "ps1" => (79, 70, 229),
            "doc" | "docx" | "pdf" | "txt" | "rtf" => (41, 98, 255),
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" => (16, 185, 129),
            "mp3" | "mp4" | "avi" | "mkv" | "wav" => (236, 72, 153),
            "zip" | "rar" | "7z" | "tar" | "gz" => (245, 158, 11),
            "rs" | "js" | "ts" | "py" | "java" | "cpp" | "h" => (139, 92, 246),
            _ => (107, 114, 128),
        };

        Some(Self::generate_colored_icon(size.pixels(), color))
    }

    /// 生成带颜色的图标 (回退方案)
    fn generate_colored_icon(size: u32, color: (u8, u8, u8)) -> Vec<u8> {
        let mut pixels = vec![0u8; (size * size * 4) as usize];

        for y in 0..size {
            for x in 0..size {
                let idx = ((y * size + x) * 4) as usize;
                if idx + 3 < pixels.len() {
                    let cx = size as f32 / 2.0;
                    let cy = size as f32 / 2.0;
                    let r = size as f32 / 2.0 - 1.0;
                    let dx = x as f32 - cx;
                    let dy = y as f32 - cy;
                    let dist = (dx * dx + dy * dy).sqrt();

                    if dist <= r {
                        pixels[idx] = color.0;
                        pixels[idx + 1] = color.1;
                        pixels[idx + 2] = color.2;
                        pixels[idx + 3] = 255;
                    } else if dist <= r + 1.0 {
                        let alpha = ((r + 1.0 - dist) * 255.0) as u8;
                        pixels[idx] = color.0;
                        pixels[idx + 1] = color.1;
                        pixels[idx + 2] = color.2;
                        pixels[idx + 3] = alpha;
                    }
                }
            }
        }

        pixels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_size_pixels() {
        assert_eq!(IconSize::Small.pixels(), 16);
        assert_eq!(IconSize::Medium.pixels(), 32);
        assert_eq!(IconSize::Large.pixels(), 48);
        assert_eq!(IconSize::ExtraLarge.pixels(), 64);
    }

    #[test]
    fn test_shell_extractor_new() {
        let extractor = ShellIconExtractor::new(NonZeroUsize::new(1000).unwrap());
        assert_eq!(extractor.pending_count(), 0);
    }

    #[test]
    fn test_generate_colored_icon() {
        let pixels = ShellIconExtractor::generate_colored_icon(16, (255, 0, 0));
        assert_eq!(pixels.len(), 16 * 16 * 4);
    }
}

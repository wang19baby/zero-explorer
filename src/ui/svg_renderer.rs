use std::sync::{Arc, Mutex};
use lru::LruCache;
use std::num::NonZeroUsize;

/// SVG图标数据
#[derive(Debug, Clone)]
pub struct SvgIcon {
    pub name: String,
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// SVG图标渲染器
/// 使用 resvg + usvg 进行真实SVG渲染
pub struct SvgIconRenderer {
    cache: Arc<Mutex<LruCache<String, Vec<u8>>>>,
}

impl SvgIconRenderer {
    pub fn new(cache_size: NonZeroUsize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(cache_size))),
        }
    }

    /// 渲染SVG图标
    pub fn render(&self, svg_data: &[u8], width: u32, height: u32) -> Option<Vec<u8>> {
        let cache_key = format!("{}x{}:{}", width, height, md5_hash(svg_data));

        // 检查缓存
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(pixels) = cache.get(&cache_key) {
                return Some(pixels.clone());
            }
        }

        // 渲染SVG
        let pixels = self.render_svg(svg_data, width, height)?;

        // 存入缓存
        {
            let mut cache = self.cache.lock().unwrap();
            cache.put(cache_key, pixels.clone());
        }

        Some(pixels)
    }

    /// 内部SVG渲染 - 使用resvg
    fn render_svg(&self, svg_data: &[u8], width: u32, height: u32) -> Option<Vec<u8>> {
        // 解析SVG
        let svg_str = std::str::from_utf8(svg_data).ok()?;

        // 使用usvg解析SVG
        let fontdb = usvg::fontdb::Database::new();
        let tree = usvg::Tree::from_str(svg_str, &usvg::Options {
            fontdb: std::sync::Arc::new(fontdb),
            ..Default::default()
        }).ok()?;

        // 计算缩放比例
        let svg_size = tree.size();
        let scale_x = width as f32 / svg_size.width();
        let scale_y = height as f32 / svg_size.height();
        let scale = scale_x.min(scale_y);

        // 使用resvg的tiny-skia类型
        let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)?;
        let mut pixmap_mut = pixmap.as_mut();

        // 清除为透明
        pixmap_mut.fill(resvg::tiny_skia::Color::TRANSPARENT);

        // 使用resvg渲染
        let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);
        resvg::render(&tree, transform, &mut pixmap_mut);

        // 转换为RGBA像素数据
        let pixels = pixmap.data().to_vec();
        Some(pixels)
    }

    /// 清空缓存
    pub fn clear_cache(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }
}

/// 简单的MD5哈希 (用于缓存键)
fn md5_hash(data: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_renderer_new() {
        let renderer = SvgIconRenderer::new(NonZeroUsize::new(100).unwrap());
        // 简单的SVG测试
        let svg = r#"<svg width="16" height="16" xmlns="http://www.w3.org/2000/svg"><rect width="16" height="16" fill="red"/></svg>"#;
        let pixels = renderer.render(svg.as_bytes(), 16, 16);
        assert!(pixels.is_some());
        assert_eq!(pixels.unwrap().len(), 16 * 16 * 4);
    }
}

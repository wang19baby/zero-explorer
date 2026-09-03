use std::sync::{Arc, Mutex};
use lru::LruCache;
use std::num::NonZeroUsize;

/// 文件夹类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FolderType {
    /// 普通文件夹
    Normal,
    /// 只读文件夹
    ReadOnly,
    /// 隐藏文件夹
    Hidden,
    /// 系统文件夹
    System,
    /// 压缩文件夹
    Compressed,
    /// 加密文件夹
    Encrypted,
    /// 快捷方式文件夹
    Shortcut,
}

/// 文件夹缩略图摊开动画类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpreadAnimation {
    /// 弧线摊开 - 层沿弧线散开
    Arc,
    /// 抽牌摊开 - 层像抽牌一样散开
    Card,
    /// 横排摊开 - 层水平排列散开
    Row,
    /// 阶梯摊开 - 层沿阶梯状散开
    Stairs,
}

impl Default for SpreadAnimation {
    fn default() -> Self {
        Self::Arc
    }
}

/// 文件夹图标合成器
/// 参考 MTT File Manager 的三层组合图标: back + front + paper
pub struct FolderIconComposer {
    cache: Arc<Mutex<LruCache<String, Vec<u8>>>>,
    /// 动画进度 (0.0 = 折叠, 1.0 = 完全展开)
    spread_progress: f32,
    /// 动画类型
    spread_animation: SpreadAnimation,
}

impl FolderIconComposer {
    pub fn new(cache_size: NonZeroUsize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(cache_size))),
            spread_progress: 0.0,
            spread_animation: SpreadAnimation::default(),
        }
    }

    /// 获取当前摊开动画进度
    pub fn spread_progress(&self) -> f32 {
        self.spread_progress
    }

    /// 设置摊开动画进度
    pub fn set_spread_progress(&mut self, progress: f32) {
        self.spread_progress = progress.clamp(0.0, 1.0);
    }

    /// 获取摊开动画类型
    pub fn spread_animation(&self) -> SpreadAnimation {
        self.spread_animation
    }

    /// 设置摊开动画类型
    pub fn set_spread_animation(&mut self, animation: SpreadAnimation) {
        self.spread_animation = animation;
    }

    /// 计算摊开时各层的偏移量
    /// 返回 (back_offset, paper_offset, front_offset) - 每个都是 (x, y)
    pub fn spread_offsets(&self, size: u32, progress: f32) -> [(f32, f32); 3] {
        if progress <= 0.0 {
            return [(0.0, 0.0), (0.0, 0.0), (0.0, 0.0)];
        }

        let s = size as f32;
        let p = progress;

        match self.spread_animation {
            SpreadAnimation::Arc => {
                // 弧线摊开 - 沿圆弧散开
                let angle_back = -0.4 * p;
                let angle_front = 0.4 * p;
                let radius = s * 0.3 * p;
                [
                    (angle_back.sin() * radius, -radius * (1.0 - angle_back.cos())),
                    (0.0, 0.0),
                    (angle_front.sin() * radius, -radius * (1.0 - angle_front.cos())),
                ]
            }
            SpreadAnimation::Card => {
                // 抽牌摊开 - 像扇形展开的扑克牌
                let offset = s * 0.25 * p;
                [
                    (-offset * 0.8, -offset * 0.4),
                    (0.0, -offset * 0.2),
                    (offset * 0.8, -offset * 0.4),
                ]
            }
            SpreadAnimation::Row => {
                // 横排摊开 - 水平排列
                let offset = s * 0.3 * p;
                [
                    (-offset, 0.0),
                    (0.0, 0.0),
                    (offset, 0.0),
                ]
            }
            SpreadAnimation::Stairs => {
                // 阶梯摊开 - 沿阶梯状上升
                let step = s * 0.2 * p;
                [
                    (-step, step),
                    (0.0, 0.0),
                    (step, -step),
                ]
            }
        }
    }

    /// 合成文件夹图标
    pub fn compose(
        &self,
        folder_type: FolderType,
        size: u32,
        paper_content: Option<&[u8]>, // 纸张内容 (可选)
    ) -> Option<Vec<u8>> {
        // 展开时不使用缓存（每帧进度不同）
        if self.spread_progress <= 0.0 {
            let cache_key = format!("{}:{}:{}", folder_type as u32, size, paper_content.is_some());
            let mut cache = self.cache.lock().unwrap();
            if let Some(pixels) = cache.get(&cache_key) {
                return Some(pixels.clone());
            }
        }

        // 合成图标
        let pixels = self.compose_icon(folder_type, size, paper_content)?;

        // 缓存折叠状态
        if self.spread_progress <= 0.0 {
            let cache_key = format!("{}:{}:{}", folder_type as u32, size, paper_content.is_some());
            let mut cache = self.cache.lock().unwrap();
            cache.put(cache_key, pixels.clone());
        }

        Some(pixels)
    }

    /// 内部图标合成
    fn compose_icon(
        &self,
        folder_type: FolderType,
        size: u32,
        paper_content: Option<&[u8]>,
    ) -> Option<Vec<u8>> {
        let mut pixels = vec![0u8; (size * size * 4) as usize];

        // 获取摊开偏移量
        let offsets = self.spread_offsets(size, self.spread_progress);

        // 1. 绘制背景 (文件夹背面) - 应用 back 偏移
        self.draw_folder_back(&mut pixels, size, folder_type, offsets[0]);

        // 2. 绘制纸张内容 (如果有) - 应用 paper 偏移
        if let Some(content) = paper_content {
            self.draw_paper_content(&mut pixels, size, content, offsets[1]);
        }

        // 3. 绘制文件夹正面 (覆盖层) - 应用 front 偏移
        self.draw_folder_front(&mut pixels, size, folder_type, offsets[2]);

        Some(pixels)
    }

    /// 绘制文件夹背面
    fn draw_folder_back(&self, pixels: &mut [u8], size: u32, folder_type: FolderType, offset: (f32, f32)) {
        let color = self.get_folder_color(folder_type);
        let ox = offset.0;
        let oy = offset.1;

        // 绘制文件夹的背面 (简单的矩形)
        for y in (size / 10)..(size * 9 / 10) {
            for x in (size / 10)..(size * 9 / 10) {
                // 应用偏移
                let sx = (x as f32 + ox) as i32;
                let sy = (y as f32 + oy) as i32;
                if sx >= 0 && sy >= 0 && (sx as u32) < size && (sy as u32) < size {
                    let idx = ((sy as u32 * size + sx as u32) * 4) as usize;
                    if idx + 3 < pixels.len() {
                        pixels[idx] = color.0;     // R
                        pixels[idx + 1] = color.1; // G
                        pixels[idx + 2] = color.2; // B
                        pixels[idx + 3] = 255;     // A
                    }
                }
            }
        }
    }

    /// 绘制纸张内容
    fn draw_paper_content(&self, pixels: &mut [u8], size: u32, content: &[u8], offset: (f32, f32)) {
        let ox = offset.0;
        let oy = offset.1;

        // 绘制白色的纸张
        for y in (size * 3 / 10)..(size * 8 / 10) {
            for x in (size * 2 / 10)..(size * 7 / 10) {
                let sx = (x as f32 + ox) as i32;
                let sy = (y as f32 + oy) as i32;
                if sx >= 0 && sy >= 0 && (sx as u32) < size && (sy as u32) < size {
                    let idx = ((sy as u32 * size + sx as u32) * 4) as usize;
                    if idx + 3 < pixels.len() {
                        pixels[idx] = 255;     // R
                        pixels[idx + 1] = 255; // G
                        pixels[idx + 2] = 255; // B
                        pixels[idx + 3] = 255; // A
                    }
                }
            }
        }

        // 简单的文本表示 (实际应渲染真实内容)
        let _ = content; // 暂时忽略内容
    }

    /// 绘制文件夹正面
    fn draw_folder_front(&self, pixels: &mut [u8], size: u32, folder_type: FolderType, offset: (f32, f32)) {
        let color = self.get_folder_color(folder_type);
        let darker = self.darker_color(color);
        let ox = offset.0;
        let oy = offset.1;

        // 绘制文件夹的正面 (覆盖在纸张上方)
        for y in (size / 10)..(size * 4 / 10) {
            for x in (size / 10)..(size * 9 / 10) {
                let sx = (x as f32 + ox) as i32;
                let sy = (y as f32 + oy) as i32;
                if sx >= 0 && sy >= 0 && (sx as u32) < size && (sy as u32) < size {
                    let idx = ((sy as u32 * size + sx as u32) * 4) as usize;
                    if idx + 3 < pixels.len() {
                        pixels[idx] = darker.0;     // R
                        pixels[idx + 1] = darker.1; // G
                        pixels[idx + 2] = darker.2; // B
                        pixels[idx + 3] = 255;      // A
                    }
                }
            }
        }

        // 绘制文件夹的标签
        for y in (size / 10)..(size * 3 / 10) {
            for x in (size / 10)..(size * 4 / 10) {
                let sx = (x as f32 + ox) as i32;
                let sy = (y as f32 + oy) as i32;
                if sx >= 0 && sy >= 0 && (sx as u32) < size && (sy as u32) < size {
                    let idx = ((sy as u32 * size + sx as u32) * 4) as usize;
                    if idx + 3 < pixels.len() {
                        pixels[idx] = color.0;     // R
                        pixels[idx + 1] = color.1; // G
                        pixels[idx + 2] = color.2; // B
                        pixels[idx + 3] = 255;     // A
                    }
                }
            }
        }
    }

    /// 获取文件夹颜色
    fn get_folder_color(&self, folder_type: FolderType) -> (u8, u8, u8) {
        match folder_type {
            FolderType::Normal => (255, 204, 0),      // 金黄色
            FolderType::ReadOnly => (180, 180, 180),   // 灰色
            FolderType::Hidden => (150, 150, 200),     // 淡紫色
            FolderType::System => (200, 100, 100),     // 红色
            FolderType::Compressed => (100, 200, 100), // 绿色
            FolderType::Encrypted => (100, 150, 200),  // 蓝色
            FolderType::Shortcut => (200, 150, 100),   // 橙色
        }
    }

    /// 使颜色变暗
    fn darker_color(&self, color: (u8, u8, u8)) -> (u8, u8, u8) {
        (
            (color.0 as f32 * 0.8) as u8,
            (color.1 as f32 * 0.8) as u8,
            (color.2 as f32 * 0.8) as u8,
        )
    }

    /// 清空缓存
    pub fn clear_cache(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_folder_type_colors() {
        let composer = FolderIconComposer::new(NonZeroUsize::new(100).unwrap());
        assert_eq!(composer.get_folder_color(FolderType::Normal), (255, 204, 0));
        assert_eq!(composer.get_folder_color(FolderType::ReadOnly), (180, 180, 180));
    }

    #[test]
    fn test_compose_icon() {
        let composer = FolderIconComposer::new(NonZeroUsize::new(100).unwrap());
        let pixels = composer.compose_icon(FolderType::Normal, 48, None).unwrap();
        assert_eq!(pixels.len(), 48 * 48 * 4);
    }
}

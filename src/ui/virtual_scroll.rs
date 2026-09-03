/// 虚拟化滚动管理器
/// 参考 MTT File Manager 的虚拟化滚动架构
/// 仅渲染可见行，避免渲染数万个不可见的文件条目
pub struct VirtualScrollManager {
    /// 总项目数
    total_items: usize,
    /// 每行高度 (像素)
    row_height: f32,
    /// 可视区域高度 (像素)
    viewport_height: f32,
    /// 当前滚动偏移 (像素)
    scroll_offset: f32,
    /// 缓冲区大小 (额外渲染的行数)
    buffer_size: usize,
}

impl VirtualScrollManager {
    pub fn new(row_height: f32, viewport_height: f32) -> Self {
        Self {
            total_items: 0,
            row_height,
            viewport_height,
            scroll_offset: 0.0,
            buffer_size: 5, // 上下各多渲染5行
        }
    }

    /// 设置总项目数
    pub fn set_total_items(&mut self, total: usize) {
        self.total_items = total;
        // 确保滚动偏移不超出范围
        self.clamp_scroll_offset();
    }

    /// 从面板快照同步初始状态
    pub fn sync_from_panel(&mut self, scroll_offset: f32, total_items: usize) {
        self.scroll_offset = scroll_offset;
        self.total_items = total_items;
        self.clamp_scroll_offset();
    }

    /// 同步滚动偏移到面板快照
    pub fn sync_to_panel(&self) -> f32 {
        self.scroll_offset
    }

    /// 设置可视区域高度
    pub fn set_viewport_height(&mut self, height: f32) {
        self.viewport_height = height;
        self.clamp_scroll_offset();
    }

    /// 设置滚动偏移
    pub fn set_scroll_offset(&mut self, offset: f32) {
        self.scroll_offset = offset.clamp(0.0, self.max_scroll_offset());
    }

    /// 滚动到指定位置
    pub fn scroll_to(&mut self, offset: f32) {
        self.set_scroll_offset(offset);
    }

    /// 滚动到指定项目
    pub fn scroll_to_item(&mut self, index: usize) {
        let offset = index as f32 * self.row_height;
        self.set_scroll_offset(offset);
    }

    /// 滚动一页
    pub fn scroll_page_up(&mut self) {
        let page_size = self.viewport_height;
        self.scroll_offset = (self.scroll_offset - page_size).max(0.0);
    }

    pub fn scroll_page_down(&mut self) {
        let page_size = self.viewport_height;
        self.scroll_offset = (self.scroll_offset + page_size).min(self.max_scroll_offset());
    }

    /// 滚动到顶部
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0.0;
    }

    /// 滚动到底部
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.max_scroll_offset();
    }

    /// 获取当前滚动偏移
    pub fn scroll_offset(&self) -> f32 {
        self.scroll_offset
    }

    /// 获取最大滚动偏移
    pub fn max_scroll_offset(&self) -> f32 {
        let total_height = self.total_items as f32 * self.row_height;
        (total_height - self.viewport_height).max(0.0)
    }

    /// 获取可见范围
    /// 返回 (first_visible, last_visible, y_offset)
    pub fn visible_range(&self) -> (usize, usize, f32) {
        let first_visible = (self.scroll_offset / self.row_height) as usize;
        let visible_count = (self.viewport_height / self.row_height) as usize + 1;

        let start = first_visible.saturating_sub(self.buffer_size);
        let end = (first_visible + visible_count + self.buffer_size).min(self.total_items);

        let y_offset = -(self.scroll_offset % self.row_height);

        (start, end, y_offset)
    }

    /// 获取可见项目索引列表
    pub fn visible_indices(&self) -> Vec<usize> {
        let (start, end, _) = self.visible_range();
        (start..end).collect()
    }

    /// 检查指定索引是否可见
    pub fn is_visible(&self, index: usize) -> bool {
        let (start, end, _) = self.visible_range();
        index >= start && index < end
    }

    /// 获取指定索引的Y坐标
    pub fn item_y_offset(&self, index: usize) -> f32 {
        index as f32 * self.row_height - self.scroll_offset
    }

    /// 确保指定项目可见
    pub fn ensure_visible(&mut self, index: usize) {
        let item_top = index as f32 * self.row_height;
        let item_bottom = item_top + self.row_height;

        if item_top < self.scroll_offset {
            // 项目在可视区域上方
            self.scroll_offset = item_top;
        } else if item_bottom > self.scroll_offset + self.viewport_height {
            // 项目在可视区域下方
            self.scroll_offset = item_bottom - self.viewport_height;
        }
    }

    /// 限制滚动偏移在有效范围内
    fn clamp_scroll_offset(&mut self) {
        self.scroll_offset = self.scroll_offset.clamp(0.0, self.max_scroll_offset());
    }

    /// 处理鼠标滚轮
    pub fn handle_scroll(&mut self, delta: f32) {
        // delta通常以像素为单位
        self.scroll_offset = (self.scroll_offset + delta).clamp(0.0, self.max_scroll_offset());
    }

    /// 处理键盘导航
    pub fn handle_key(&mut self, key: NavigationKey) {
        match key {
            NavigationKey::Up => {
                self.scroll_offset = (self.scroll_offset - self.row_height).max(0.0);
            }
            NavigationKey::Down => {
                self.scroll_offset = (self.scroll_offset + self.row_height)
                    .min(self.max_scroll_offset());
            }
            NavigationKey::PageUp => self.scroll_page_up(),
            NavigationKey::PageDown => self.scroll_page_down(),
            NavigationKey::Home => self.scroll_to_top(),
            NavigationKey::End => self.scroll_to_bottom(),
        }
    }
}

/// 导航键
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationKey {
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
}

impl Default for VirtualScrollManager {
    fn default() -> Self {
        Self::new(24.0, 600.0) // 默认24px行高, 600px视口高度
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visible_range() {
        let mut manager = VirtualScrollManager::new(24.0, 100.0);
        manager.set_total_items(1000);

        let (start, end, _) = manager.visible_range();
        assert_eq!(start, 0);
        assert!(end > 0);
        assert!(end <= 1000);
    }

    #[test]
    fn test_scroll_to_item() {
        let mut manager = VirtualScrollManager::new(24.0, 100.0);
        manager.set_total_items(1000);

        manager.scroll_to_item(50);
        assert!(manager.scroll_offset() > 0.0);
    }

    #[test]
    fn test_ensure_visible() {
        let mut manager = VirtualScrollManager::new(24.0, 100.0);
        manager.set_total_items(1000);

        manager.ensure_visible(100);
        assert!(manager.is_visible(100));
    }
}

use super::components::Rect;
use std::time::{Duration, Instant};

/// 按钮提示项
#[derive(Debug, Clone)]
pub struct TooltipItem {
    /// 提示文本（第一行：功能描述）
    pub text: String,
    /// 快捷键/手势（第二行）
    pub shortcut: Option<String>,
    /// 鼠标手势（可选）
    pub gesture: Option<String>,
}

impl TooltipItem {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            shortcut: None,
            gesture: None,
        }
    }

    pub fn with_shortcut(text: &str, shortcut: &str) -> Self {
        Self {
            text: text.to_string(),
            shortcut: Some(shortcut.to_string()),
            gesture: None,
        }
    }

    pub fn with_gesture(text: &str, gesture: &str) -> Self {
        Self {
            text: text.to_string(),
            shortcut: None,
            gesture: Some(gesture.to_string()),
        }
    }

    pub fn with_both(text: &str, shortcut: &str, gesture: &str) -> Self {
        Self {
            text: text.to_string(),
            shortcut: Some(shortcut.to_string()),
            gesture: Some(gesture.to_string()),
        }
    }
}

/// 按钮提示系统 - 参考 Tessoa 的按钮提示
#[derive(Debug)]
pub struct TooltipSystem {
    /// 当前显示的提示
    current_tooltip: Option<TooltipItem>,
    /// 提示位置
    tooltip_bounds: Rect,
    /// 提示显示时间
    show_time: Option<Instant>,
    /// 提示延迟（鼠标悬停多久后显示）
    delay: Duration,
    /// 提示最大宽度
    max_width: f32,
    /// 提示背景颜色
    background_color: [f32; 4],
    /// 提示文字颜色
    text_color: [f32; 4],
    /// 提示边框颜色
    border_color: [f32; 4],
    /// 提示圆角
    border_radius: f32,
    /// 提示内边距
    padding: f32,
    /// 提示行高
    line_height: f32,
}

impl TooltipSystem {
    pub fn new() -> Self {
        Self {
            current_tooltip: None,
            tooltip_bounds: Rect::default(),
            show_time: None,
            delay: Duration::from_millis(400),
            max_width: 300.0,
            background_color: [0.1, 0.1, 0.1, 0.95],
            text_color: [0.9, 0.9, 0.9, 1.0],
            border_color: [0.3, 0.3, 0.3, 1.0],
            border_radius: 4.0,
            padding: 8.0,
            line_height: 20.0,
        }
    }

    /// 显示提示
    pub fn show(&mut self, tooltip: TooltipItem, x: f32, y: f32) {
        self.current_tooltip = Some(tooltip);
        self.show_time = Some(Instant::now());
        self.update_bounds(x, y);
    }

    /// 隐藏提示
    pub fn hide(&mut self) {
        self.current_tooltip = None;
        self.show_time = None;
    }

    /// 更新提示位置
    pub fn update_bounds(&mut self, x: f32, y: f32) {
        if let Some(ref tooltip) = self.current_tooltip {
            let text_width = self.estimate_text_width(&tooltip.text);
            let shortcut_width = tooltip.shortcut.as_ref().map(|s| self.estimate_text_width(s)).unwrap_or(0.0);
            let gesture_width = tooltip.gesture.as_ref().map(|g| self.estimate_text_width(g)).unwrap_or(0.0);

            let content_width = text_width.max(shortcut_width).max(gesture_width);
            let width = (content_width + self.padding * 2.0).min(self.max_width);

            let lines = 1
                + tooltip.shortcut.as_ref().map_or(0, |_| 1)
                + tooltip.gesture.as_ref().map_or(0, |_| 1);
            let height = (lines as f32) * self.line_height + self.padding * 2.0;

            self.tooltip_bounds = Rect::new(x, y - height - 5.0, width, height);
        }
    }

    /// 估算文本宽度
    fn estimate_text_width(&self, text: &str) -> f32 {
        // 简单估算：每个字符约 8 像素宽
        text.len() as f32 * 8.0
    }

    /// 检查是否应该显示提示
    pub fn should_show(&self) -> bool {
        if let Some(show_time) = self.show_time {
            show_time.elapsed() >= self.delay
        } else {
            false
        }
    }

    /// 获取当前提示
    pub fn current_tooltip(&self) -> Option<&TooltipItem> {
        self.current_tooltip.as_ref()
    }

    /// 获取提示边界
    pub fn tooltip_bounds(&self) -> &Rect {
        &self.tooltip_bounds
    }

    /// 检查点是否在提示内
    pub fn contains(&self, x: f32, y: f32) -> bool {
        self.tooltip_bounds.contains(x, y)
    }

    /// 获取提示的行数
    pub fn line_count(&self) -> usize {
        if let Some(ref tooltip) = self.current_tooltip {
            let mut count = 1;
            if tooltip.shortcut.is_some() {
                count += 1;
            }
            if tooltip.gesture.is_some() {
                count += 1;
            }
            count
        } else {
            0
        }
    }

    /// 获取提示的背景颜色
    pub fn background_color(&self) -> [f32; 4] {
        self.background_color
    }

    /// 获取提示的文字颜色
    pub fn text_color(&self) -> [f32; 4] {
        self.text_color
    }

    /// 获取提示的边框颜色
    pub fn border_color(&self) -> [f32; 4] {
        self.border_color
    }

    /// 获取提示的圆角
    pub fn border_radius(&self) -> f32 {
        self.border_radius
    }

    /// 获取提示的内边距
    pub fn padding(&self) -> f32 {
        self.padding
    }

    /// 获取提示的行高
    pub fn line_height(&self) -> f32 {
        self.line_height
    }
}

impl Default for TooltipSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// 按钮提示配置
#[derive(Debug, Clone)]
pub struct ButtonTooltipConfig {
    /// 按钮 ID
    pub button_id: String,
    /// 提示项
    pub tooltip: TooltipItem,
    /// 提示位置偏移
    pub offset: (f32, f32),
}

impl ButtonTooltipConfig {
    pub fn new(button_id: &str, tooltip: TooltipItem) -> Self {
        Self {
            button_id: button_id.to_string(),
            tooltip,
            offset: (0.0, -10.0),
        }
    }

    pub fn with_offset(mut self, x: f32, y: f32) -> Self {
        self.offset = (x, y);
        self
    }
}

/// 按钮提示管理器
#[derive(Debug)]
pub struct ButtonTooltipManager {
    /// 所有按钮的提示配置
    configs: Vec<ButtonTooltipConfig>,
    /// 当前显示的提示系统
    tooltip_system: TooltipSystem,
    /// 当前 hover 的按钮 ID
    hovered_button: Option<String>,
}

impl ButtonTooltipManager {
    pub fn new() -> Self {
        Self {
            configs: Vec::new(),
            tooltip_system: TooltipSystem::new(),
            hovered_button: None,
        }
    }

    /// 注册按钮提示
    pub fn register(&mut self, config: ButtonTooltipConfig) {
        self.configs.push(config);
    }

    /// 注册多个按钮提示
    pub fn register_many(&mut self, configs: Vec<ButtonTooltipConfig>) {
        for config in configs {
            self.register(config);
        }
    }

    /// 更新 hover 状态
    pub fn update_hover(&mut self, button_id: Option<&str>, x: f32, y: f32) {
        if let Some(id) = button_id {
            if self.hovered_button.as_deref() != Some(id) {
                // 新的按钮 hover
                self.hovered_button = Some(id.to_string());
                self.tooltip_system.hide();

                // 查找对应的提示配置
                if let Some(config) = self.configs.iter().find(|c| c.button_id == id) {
                    let tooltip = config.tooltip.clone();
                    self.tooltip_system.show(
                        tooltip,
                        x + config.offset.0,
                        y + config.offset.1,
                    );
                }
            }
        } else {
            // 没有按钮 hover
            self.hovered_button = None;
            self.tooltip_system.hide();
        }
    }

    /// 获取当前提示系统
    pub fn tooltip_system(&self) -> &TooltipSystem {
        &self.tooltip_system
    }

    /// 获取当前 hover 的按钮 ID
    pub fn hovered_button(&self) -> Option<&str> {
        self.hovered_button.as_deref()
    }

    /// 清除所有配置
    pub fn clear(&mut self) {
        self.configs.clear();
        self.tooltip_system.hide();
        self.hovered_button = None;
    }
}

impl Default for ButtonTooltipManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tooltip_item_new() {
        let item = TooltipItem::new("测试");
        assert_eq!(item.text, "测试");
        assert!(item.shortcut.is_none());
        assert!(item.gesture.is_none());
    }

    #[test]
    fn test_tooltip_item_with_shortcut() {
        let item = TooltipItem::with_shortcut("复制", "Ctrl+C");
        assert_eq!(item.text, "复制");
        assert_eq!(item.shortcut.as_deref(), Some("Ctrl+C"));
        assert!(item.gesture.is_none());
    }

    #[test]
    fn test_tooltip_item_with_gesture() {
        let item = TooltipItem::with_gesture("打开", "中键");
        assert_eq!(item.text, "打开");
        assert!(item.shortcut.is_none());
        assert_eq!(item.gesture.as_deref(), Some("中键"));
    }

    #[test]
    fn test_tooltip_item_with_both() {
        let item = TooltipItem::with_both("后退", "Alt+←", "侧键 1");
        assert_eq!(item.text, "后退");
        assert_eq!(item.shortcut.as_deref(), Some("Alt+←"));
        assert_eq!(item.gesture.as_deref(), Some("侧键 1"));
    }

    #[test]
    fn test_tooltip_system_new() {
        let system = TooltipSystem::new();
        assert!(system.current_tooltip().is_none());
        assert!(!system.should_show());
    }

    #[test]
    fn test_tooltip_system_show_hide() {
        let mut system = TooltipSystem::new();
        let tooltip = TooltipItem::new("测试提示");

        system.show(tooltip, 100.0, 100.0);
        assert!(system.current_tooltip().is_some());

        system.hide();
        assert!(system.current_tooltip().is_none());
    }

    #[test]
    fn test_tooltip_system_bounds() {
        let mut system = TooltipSystem::new();
        let tooltip = TooltipItem::new("测试");

        system.show(tooltip, 100.0, 100.0);
        let bounds = system.tooltip_bounds();

        // 提示应该在鼠标上方
        assert!(bounds.y < 100.0);
        // 提示宽度应该大于 0
        assert!(bounds.width > 0.0);
        // 提示高度应该大于 0
        assert!(bounds.height > 0.0);
    }

    #[test]
    fn test_tooltip_system_line_count() {
        let mut system = TooltipSystem::new();

        // 无提示
        assert_eq!(system.line_count(), 0);

        // 单行提示
        let tooltip = TooltipItem::new("测试");
        system.show(tooltip, 0.0, 0.0);
        assert_eq!(system.line_count(), 1);

        // 双行提示
        let tooltip = TooltipItem::with_shortcut("测试", "Ctrl+T");
        system.show(tooltip, 0.0, 0.0);
        assert_eq!(system.line_count(), 2);

        // 三行提示
        let tooltip = TooltipItem::with_both("测试", "Ctrl+T", "中键");
        system.show(tooltip, 0.0, 0.0);
        assert_eq!(system.line_count(), 3);
    }

    #[test]
    fn test_tooltip_manager_new() {
        let manager = ButtonTooltipManager::new();
        assert!(manager.hovered_button().is_none());
    }

    #[test]
    fn test_tooltip_manager_register() {
        let mut manager = ButtonTooltipManager::new();
        let config = ButtonTooltipConfig::new(
            "copy",
            TooltipItem::with_shortcut("复制", "Ctrl+C"),
        );
        manager.register(config);

        // 注册后应该能找到
        assert!(!manager.configs.is_empty());
    }

    #[test]
    fn test_tooltip_manager_hover() {
        let mut manager = ButtonTooltipManager::new();
        let config = ButtonTooltipConfig::new(
            "copy",
            TooltipItem::with_shortcut("复制", "Ctrl+C"),
        );
        manager.register(config);

        // hover 到按钮上
        manager.update_hover(Some("copy"), 100.0, 100.0);
        assert_eq!(manager.hovered_button(), Some("copy"));

        // hover 离开
        manager.update_hover(None, 200.0, 200.0);
        assert!(manager.hovered_button().is_none());
    }

    #[test]
    fn test_tooltip_manager_clear() {
        let mut manager = ButtonTooltipManager::new();
        let config = ButtonTooltipConfig::new(
            "copy",
            TooltipItem::with_shortcut("复制", "Ctrl+C"),
        );
        manager.register(config);

        manager.update_hover(Some("copy"), 100.0, 100.0);
        assert!(!manager.configs.is_empty());

        manager.clear();
        assert!(manager.configs.is_empty());
        assert!(manager.hovered_button().is_none());
    }
}

use super::components::{Component, ComponentState, Rect};
use super::text_render_settings::TextRenderSettings;
use super::theme::ThemeManager;

/// 设置面板 - 参考 Tessoa 的设置界面
#[derive(Debug)]
pub struct SettingsPanel {
    id: String,
    bounds: Rect,
    state: ComponentState,
    visible: bool,
    /// 当前选中的设置组
    selected_group: SettingsGroup,
    /// 文字渲染设置
    text_render_settings: TextRenderSettings,
    /// 主题管理器引用
    theme_manager: Option<*mut ThemeManager>,
    /// 搜索框内容
    search_query: String,
    /// 设置组列表
    groups: Vec<SettingsGroupItem>,
}

/// 设置组
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsGroup {
    General,
    Appearance,
    Sidebar,
    View,
    Tabs,
    Input,
    Display,
    Notifications,
    Maintenance,
    About,
}

/// 设置组项
#[derive(Debug, Clone)]
pub struct SettingsGroupItem {
    pub group: SettingsGroup,
    pub name: String,
    pub icon: Option<String>,
}

impl SettingsGroupItem {
    pub fn new(group: SettingsGroup, name: &str) -> Self {
        Self {
            group,
            name: name.to_string(),
            icon: None,
        }
    }
}

impl SettingsPanel {
    pub fn new() -> Self {
        let groups = vec![
            SettingsGroupItem::new(SettingsGroup::General, "通用"),
            SettingsGroupItem::new(SettingsGroup::Appearance, "外观"),
            SettingsGroupItem::new(SettingsGroup::Sidebar, "侧边栏"),
            SettingsGroupItem::new(SettingsGroup::View, "视图"),
            SettingsGroupItem::new(SettingsGroup::Tabs, "标签页"),
            SettingsGroupItem::new(SettingsGroup::Input, "操作"),
            SettingsGroupItem::new(SettingsGroup::Display, "显示与性能"),
            SettingsGroupItem::new(SettingsGroup::Notifications, "通知"),
            SettingsGroupItem::new(SettingsGroup::Maintenance, "维护"),
            SettingsGroupItem::new(SettingsGroup::About, "关于"),
        ];

        Self {
            id: "settings_panel".to_string(),
            bounds: Rect::default(),
            state: ComponentState::Normal,
            visible: false,
            selected_group: SettingsGroup::General,
            text_render_settings: TextRenderSettings::default(),
            theme_manager: None,
            search_query: String::new(),
            groups,
        }
    }

    /// 打开设置面板
    pub fn open(&mut self) {
        self.visible = true;
    }

    /// 关闭设置面板
    pub fn close(&mut self) {
        self.visible = false;
    }

    /// 获取文字渲染设置
    pub fn text_render_settings(&self) -> &TextRenderSettings {
        &self.text_render_settings
    }

    /// 获取文字渲染设置（可变）
    pub fn text_render_settings_mut(&mut self) -> &mut TextRenderSettings {
        &mut self.text_render_settings
    }

    /// 设置主题管理器
    pub fn set_theme_manager(&mut self, manager: *mut ThemeManager) {
        self.theme_manager = Some(manager);
    }

    /// 获取当前选中的设置组
    pub fn selected_group(&self) -> SettingsGroup {
        self.selected_group
    }

    /// 选择设置组
    pub fn select_group(&mut self, group: SettingsGroup) {
        self.selected_group = group;
    }

    /// 获取设置组列表
    pub fn groups(&self) -> &[SettingsGroupItem] {
        &self.groups
    }

    /// 设置搜索查询
    pub fn set_search_query(&mut self, query: &str) {
        self.search_query = query.to_string();
    }

    /// 获取搜索查询
    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    /// 过滤设置组
    pub fn filtered_groups(&self) -> Vec<&SettingsGroupItem> {
        if self.search_query.is_empty() {
            self.groups.iter().collect()
        } else {
            self.groups
                .iter()
                .filter(|g| g.name.to_lowercase().contains(&self.search_query.to_lowercase()))
                .collect()
        }
    }

    /// 获取设置组的边界
    pub fn group_bounds(&self, index: usize) -> Option<Rect> {
        if index >= self.groups.len() {
            return None;
        }

        let group_height = 32.0;
        let x = self.bounds.x;
        let y = self.bounds.y + 60.0 + (index as f32) * group_height;

        Some(Rect::new(x, y, 160.0, group_height))
    }

    /// 获取设置内容区域的边界
    pub fn content_bounds(&self) -> Rect {
        Rect::new(
            self.bounds.x + 170.0,
            self.bounds.y + 60.0,
            self.bounds.width - 180.0,
            self.bounds.height - 70.0,
        )
    }

    /// 获取搜索框的边界
    pub fn search_bounds(&self) -> Rect {
        Rect::new(
            self.bounds.x + 10.0,
            self.bounds.y + 10.0,
            self.bounds.width - 20.0,
            36.0,
        )
    }

    /// 获取关闭按钮的边界
    pub fn close_button_bounds(&self) -> Rect {
        Rect::new(
            self.bounds.x + self.bounds.width - 40.0,
            self.bounds.y + 10.0,
            30.0,
            30.0,
        )
    }

    /// 渲染外观设置
    pub fn render_appearance_settings(&self) -> Vec<SettingsItem> {
        let mut items = Vec::new();
        let content = self.content_bounds();
        let item_height = 40.0;
        let mut y = content.y + 10.0;

        // 主题选择
        items.push(SettingsItem::new(
            "theme",
            "主题",
            SettingsItemType::Dropdown,
            Rect::new(content.x + 10.0, y, content.width - 20.0, item_height),
        ));
        y += item_height + 10.0;

        // 字体大小
        items.push(SettingsItem::new(
            "font_size",
            "字体大小",
            SettingsItemType::Slider {
                min: 10.0,
                max: 24.0,
                value: 14.0,
            },
            Rect::new(content.x + 10.0, y, content.width - 20.0, item_height),
        ));
        y += item_height + 10.0;

        // 间隔宽度
        items.push(SettingsItem::new(
            "spacing",
            "间隔宽度",
            SettingsItemType::Slider {
                min: 0.0,
                max: 16.0,
                value: 4.0,
            },
            Rect::new(content.x + 10.0, y, content.width - 20.0, item_height),
        ));
        y += item_height + 10.0;

        // 圆角
        items.push(SettingsItem::new(
            "border_radius",
            "圆角",
            SettingsItemType::Slider {
                min: 0.0,
                max: 16.0,
                value: 4.0,
            },
            Rect::new(content.x + 10.0, y, content.width - 20.0, item_height),
        ));
        y += item_height + 10.0;

        // 斑马纹
        items.push(SettingsItem::new(
            "zebra_striping",
            "斑马纹（隔行交错色）",
            SettingsItemType::Toggle,
            Rect::new(content.x + 10.0, y, content.width - 20.0, item_height),
        ));
        y += item_height + 10.0;

        // 扁平目录图标
        items.push(SettingsItem::new(
            "flat_folder_icons",
            "扁平目录图标",
            SettingsItemType::Toggle,
            Rect::new(content.x + 10.0, y, content.width - 20.0, item_height),
        ));
        y += item_height + 10.0;

        // 省略号放中间
        items.push(SettingsItem::new(
            "ellipsis_middle",
            "省略号放中间",
            SettingsItemType::Toggle,
            Rect::new(content.x + 10.0, y, content.width - 20.0, item_height),
        ));

        items
    }

    /// 渲染文字渲染设置
    pub fn render_text_render_settings(&self) -> Vec<SettingsItem> {
        let mut items = Vec::new();
        let content = self.content_bounds();
        let item_height = 40.0;
        let mut y = content.y + 10.0;

        // 亚像素定位
        items.push(SettingsItem::new(
            "subpixel_positioning",
            "亚像素定位",
            SettingsItemType::Toggle,
            Rect::new(content.x + 10.0, y, content.width - 20.0, item_height),
        ));
        y += item_height + 10.0;

        // LCD 亚像素抗锯齿
        items.push(SettingsItem::new(
            "lcd_subpixel_aa",
            "LCD 亚像素抗锯齿",
            SettingsItemType::Toggle,
            Rect::new(content.x + 10.0, y, content.width - 20.0, item_height),
        ));
        y += item_height + 10.0;

        // 字形对齐像素格
        items.push(SettingsItem::new(
            "glyph_hinting",
            "字形对齐像素格",
            SettingsItemType::Dropdown,
            Rect::new(content.x + 10.0, y, content.width - 20.0, item_height),
        ));
        y += item_height + 10.0;

        // 文字 Gamma
        items.push(SettingsItem::new(
            "text_gamma",
            "文字 Gamma",
            SettingsItemType::Slider {
                min: 0.60,
                max: 1.60,
                value: self.text_render_settings.text_gamma,
            },
            Rect::new(content.x + 10.0, y, content.width - 20.0, item_height),
        ));
        y += item_height + 10.0;

        // 经典文字渲染引擎 (Windows only)
        #[cfg(target_os = "windows")]
        {
            items.push(SettingsItem::new(
                "classic_text_engine",
                "经典文字渲染引擎",
                SettingsItemType::Dropdown,
                Rect::new(content.x + 10.0, y, content.width - 20.0, item_height),
            ));
        }

        items
    }
}

impl Default for SettingsPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for SettingsPanel {
    fn id(&self) -> &str {
        &self.id
    }

    fn bounds(&self) -> &Rect {
        &self.bounds
    }

    fn bounds_mut(&mut self) -> &mut Rect {
        &mut self.bounds
    }

    fn state(&self) -> &ComponentState {
        &self.state
    }

    fn set_state(&mut self, state: ComponentState) {
        self.state = state;
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn handle_mouse_move(&mut self, x: f32, y: f32) -> bool {
        if self.bounds.contains(x, y) {
            self.set_state(ComponentState::Hovered);
            true
        } else {
            if *self.state() == ComponentState::Hovered {
                self.set_state(ComponentState::Normal);
            }
            false
        }
    }

    fn handle_mouse_button_down(&mut self, x: f32, y: f32) -> bool {
        if self.bounds.contains(x, y) {
            self.set_state(ComponentState::Pressed);

            // 检查关闭按钮
            let close_bounds = self.close_button_bounds();
            if close_bounds.contains(x, y) {
                self.close();
                return true;
            }

            // 检查设置组选择
            for (i, group) in self.groups.iter().enumerate() {
                if let Some(bounds) = self.group_bounds(i) {
                    if bounds.contains(x, y) {
                        self.select_group(group.group);
                        return true;
                    }
                }
            }

            true
        } else {
            false
        }
    }

    fn handle_mouse_button_up(&mut self, _x: f32, _y: f32) -> bool {
        if *self.state() == ComponentState::Pressed {
            self.set_state(ComponentState::Hovered);
            true
        } else {
            false
        }
    }

    fn handle_key_down(&mut self, key: u32) -> bool {
        match key {
            27 => {
                // ESC 关闭设置面板
                self.close();
                true
            }
            _ => false,
        }
    }

    fn handle_char_input(&mut self, ch: char) -> bool {
        if self.visible {
            match ch {
                '\u{0008}' => {
                    // Backspace
                    self.search_query.pop();
                    true
                }
                '\u{007F}' => {
                    // Delete
                    self.search_query.clear();
                    true
                }
                _ if ch.is_alphanumeric() || ch == ' ' || ch == '_' => {
                    self.search_query.push(ch);
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }
}

/// 设置项
#[derive(Debug, Clone)]
pub struct SettingsItem {
    pub id: String,
    pub label: String,
    pub item_type: SettingsItemType,
    pub bounds: Rect,
}

impl SettingsItem {
    pub fn new(id: &str, label: &str, item_type: SettingsItemType, bounds: Rect) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            item_type,
            bounds,
        }
    }
}

/// 设置项类型
#[derive(Debug, Clone)]
pub enum SettingsItemType {
    Toggle,
    Slider { min: f32, max: f32, value: f32 },
    Dropdown,
    TextInput,
    Button,
    ColorPicker,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_panel_new() {
        let panel = SettingsPanel::new();
        assert_eq!(panel.id(), "settings_panel");
        assert!(!panel.visible);
        assert_eq!(panel.selected_group(), SettingsGroup::General);
    }

    #[test]
    fn test_settings_panel_open_close() {
        let mut panel = SettingsPanel::new();
        panel.open();
        assert!(panel.visible);

        panel.close();
        assert!(!panel.visible);
    }

    #[test]
    fn test_settings_panel_select_group() {
        let mut panel = SettingsPanel::new();
        panel.select_group(SettingsGroup::Appearance);
        assert_eq!(panel.selected_group(), SettingsGroup::Appearance);
    }

    #[test]
    fn test_settings_panel_search() {
        let mut panel = SettingsPanel::new();
        panel.set_search_query("外观");
        
        let filtered = panel.filtered_groups();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "外观");
    }

    #[test]
    fn test_settings_panel_text_render_settings() {
        let mut panel = SettingsPanel::new();
        
        // 默认设置
        assert!(panel.text_render_settings().subpixel_positioning);
        assert!(!panel.text_render_settings().lcd_subpixel_aa);
        assert_eq!(panel.text_render_settings().text_gamma, 1.0);

        // 修改设置
        panel.text_render_settings_mut().set_gamma(1.2);
        assert_eq!(panel.text_render_settings().text_gamma, 1.2);
    }

    #[test]
    fn test_settings_panel_bounds() {
        let mut panel = SettingsPanel::new();
        panel.bounds = Rect::new(100.0, 100.0, 800.0, 600.0);

        // 搜索框边界
        let search_bounds = panel.search_bounds();
        assert!(search_bounds.x > panel.bounds.x);

        // 关闭按钮边界
        let close_bounds = panel.close_button_bounds();
        assert!(close_bounds.x > panel.bounds.x);

        // 内容区域边界
        let content_bounds = panel.content_bounds();
        assert!(content_bounds.x > panel.bounds.x + 160.0);
    }

    #[test]
    fn test_settings_panel_keyboard() {
        let mut panel = SettingsPanel::new();
        panel.open();

        // ESC 关闭
        assert!(panel.handle_key_down(27));
        assert!(!panel.visible);
    }

    #[test]
    fn test_settings_panel_char_input() {
        let mut panel = SettingsPanel::new();
        panel.open();

        // 输入字符
        assert!(panel.handle_char_input('外'));
        assert_eq!(panel.search_query(), "外");

        // 退格
        assert!(panel.handle_char_input('\u{0008}'));
        assert_eq!(panel.search_query(), "");

        // 清除
        panel.set_search_query("测试");
        assert!(panel.handle_char_input('\u{007F}'));
        assert_eq!(panel.search_query(), "");
    }
}

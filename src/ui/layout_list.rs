use super::components::{Component, ComponentState, Rect};
use crate::core::state::LayoutState;
use std::time::{Duration, Instant};

/// 布局列表项的 hover 状态
#[derive(Debug, Clone)]
pub struct LayoutItemHover {
    /// 是否显示操作按钮
    pub show_actions: bool,
    /// 是否显示锁定图标
    pub show_lock: bool,
    /// 是否显示更多菜单按钮
    pub show_menu: bool,
    /// hover 开始时间
    pub hover_start: Option<Instant>,
}

impl Default for LayoutItemHover {
    fn default() -> Self {
        Self {
            show_actions: false,
            show_lock: false,
            show_menu: false,
            hover_start: None,
        }
    }
}

/// 布局列表项
#[derive(Debug, Clone)]
pub struct LayoutListItem {
    /// 布局数据
    pub layout: LayoutState,
    /// 布局索引
    pub index: usize,
    /// 是否是当前活跃布局
    pub is_active: bool,
    /// hover 状态
    pub hover: LayoutItemHover,
    /// 行边界
    pub bounds: Rect,
}

impl LayoutListItem {
    pub fn new(layout: LayoutState, index: usize, is_active: bool) -> Self {
        Self {
            layout,
            index,
            is_active,
            hover: LayoutItemHover::default(),
            bounds: Rect::default(),
        }
    }

    /// 获取显示名称
    pub fn display_name(&self) -> &str {
        if self.layout.name.is_empty() {
            "(当前布局)"
        } else {
            &self.layout.name
        }
    }

    /// 是否是未命名布局
    pub fn is_unnamed(&self) -> bool {
        self.layout.name.is_empty()
    }

    /// 是否是锁定的布局
    pub fn is_locked(&self) -> bool {
        self.layout.is_locked
    }
}

/// 布局列表 - 参考 Tessoa 的侧栏布局分组
#[derive(Debug)]
pub struct LayoutList {
    id: String,
    bounds: Rect,
    state: ComponentState,
    visible: bool,
    /// 布局列表项
    items: Vec<LayoutListItem>,
    /// 当前 hover 的项索引
    hovered_index: Option<usize>,
    /// 未命名布局保留槽
    unnamed_layout: Option<LayoutState>,
    /// 上次未命名的布局
    last_unnamed: Option<LayoutState>,
    /// hover 延迟时间
    hover_delay: Duration,
}

impl LayoutList {
    pub fn new() -> Self {
        Self {
            id: "layout_list".to_string(),
            bounds: Rect::default(),
            state: ComponentState::Normal,
            visible: true,
            items: Vec::new(),
            hovered_index: None,
            unnamed_layout: None,
            last_unnamed: None,
            hover_delay: Duration::from_millis(200),
        }
    }

    /// 设置布局列表
    pub fn set_layouts(&mut self, layouts: Vec<LayoutState>, active_index: Option<usize>) {
        self.items = layouts
            .into_iter()
            .enumerate()
            .map(|(i, layout)| LayoutListItem::new(layout, i, Some(i) == active_index))
            .collect();
    }

    /// 设置未命名布局
    pub fn set_unnamed_layout(&mut self, layout: Option<LayoutState>) {
        self.unnamed_layout = layout;
    }

    /// 设置上次未命名的布局
    pub fn set_last_unnamed(&mut self, layout: Option<LayoutState>) {
        self.last_unnamed = layout;
    }

    /// 获取所有布局项
    pub fn items(&self) -> &[LayoutListItem] {
        &self.items
    }

    /// 获取当前 hover 的项
    pub fn hovered_item(&self) -> Option<&LayoutListItem> {
        self.hovered_index.and_then(|i| self.items.get(i))
    }

    /// 获取当前 hover 的项（可变）
    pub fn hovered_item_mut(&mut self) -> Option<&mut LayoutListItem> {
        self.hovered_index.and_then(|i| self.items.get_mut(i))
    }

    /// 更新 hover 状态
    pub fn update_hover(&mut self, x: f32, y: f32) -> Option<usize> {
        let item_height = 32.0;
        let item_index = ((y - self.bounds.y) / item_height) as usize;

        if item_index < self.items.len() && self.bounds.contains(x, y) {
            self.hovered_index = Some(item_index);

            // 更新 hover 状态
            let now = Instant::now();
            for (i, item) in self.items.iter_mut().enumerate() {
                if i == item_index {
                    if item.hover.hover_start.is_none() {
                        item.hover.hover_start = Some(now);
                    }
                    let elapsed = now.duration_since(item.hover.hover_start.unwrap());
                    if elapsed >= self.hover_delay {
                        item.hover.show_actions = true;
                        item.hover.show_lock = true;
                        item.hover.show_menu = true;
                    }
                } else {
                    item.hover = LayoutItemHover::default();
                }
            }

            Some(item_index)
        } else {
            // 清除所有 hover 状态
            for item in &mut self.items {
                item.hover = LayoutItemHover::default();
            }
            self.hovered_index = None;
            None
        }
    }

    /// 获取锁定按钮的边界
    pub fn lock_button_bounds(&self, item: &LayoutListItem) -> Rect {
        let row_right = self.bounds.x + self.bounds.width;
        Rect::new(row_right - 60.0, item.bounds.y + 4.0, 24.0, 24.0)
    }

    /// 获取重新加载按钮的边界
    pub fn reload_button_bounds(&self, item: &LayoutListItem) -> Rect {
        let row_right = self.bounds.x + self.bounds.width;
        Rect::new(row_right - 85.0, item.bounds.y + 4.0, 24.0, 24.0)
    }

    /// 获取更多菜单按钮的边界
    pub fn menu_button_bounds(&self, item: &LayoutListItem) -> Rect {
        let row_right = self.bounds.x + self.bounds.width;
        Rect::new(row_right - 32.0, item.bounds.y + 4.0, 24.0, 24.0)
    }

    /// 获取布局行的边界
    pub fn item_bounds(&self, index: usize) -> Option<Rect> {
        self.items.get(index).map(|item| item.bounds.clone())
    }

    /// 检查点击是否在锁定按钮上
    pub fn hit_test_lock(&self, x: f32, y: f32) -> Option<usize> {
        for (i, item) in self.items.iter().enumerate() {
            let bounds = self.lock_button_bounds(item);
            if bounds.contains(x, y) {
                return Some(i);
            }
        }
        None
    }

    /// 检查点击是否在重新加载按钮上
    pub fn hit_test_reload(&self, x: f32, y: f32) -> Option<usize> {
        for (i, item) in self.items.iter().enumerate() {
            let bounds = self.reload_button_bounds(item);
            if bounds.contains(x, y) {
                return Some(i);
            }
        }
        None
    }

    /// 检查点击是否在更多菜单按钮上
    pub fn hit_test_menu(&self, x: f32, y: f32) -> Option<usize> {
        for (i, item) in self.items.iter().enumerate() {
            let bounds = self.menu_button_bounds(item);
            if bounds.contains(x, y) {
                return Some(i);
            }
        }
        None
    }
}

impl Default for LayoutList {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for LayoutList {
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
            self.update_hover(x, y);
            true
        } else {
            if *self.state() == ComponentState::Hovered {
                self.set_state(ComponentState::Normal);
                self.hovered_index = None;
                for item in &mut self.items {
                    item.hover = LayoutItemHover::default();
                }
            }
            false
        }
    }

    fn handle_mouse_button_down(&mut self, x: f32, y: f32) -> bool {
        if self.bounds.contains(x, y) {
            self.set_state(ComponentState::Pressed);

            // 检查点击的行
            let item_height = 32.0;
            let item_index = ((y - self.bounds.y) / item_height) as usize;

            if item_index < self.items.len() {
                // 检查是否点击了操作按钮
                let item = &self.items[item_index];

                // 锁定按钮
                let lock_bounds = self.lock_button_bounds(item);
                if lock_bounds.contains(x, y) {
                    return true; // 返回 true 表示点击了锁定按钮
                }

                // 重新加载按钮
                let reload_bounds = self.reload_button_bounds(item);
                if reload_bounds.contains(x, y) {
                    return true; // 返回 true 表示点击了重新加载按钮
                }

                // 更多菜单按钮
                let menu_bounds = self.menu_button_bounds(item);
                if menu_bounds.contains(x, y) {
                    return true; // 返回 true 表示点击了更多菜单按钮
                }

                // 点击行本身
                return true;
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
                // ESC 关闭布局列表
                self.visible = false;
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::state::LayoutMode;

    fn create_test_layout(name: &str, is_locked: bool) -> LayoutState {
        LayoutState {
            name: name.to_string(),
            mode: LayoutMode::Single,
            panels: vec![],
            is_locked,
            is_saved: true,
            created_at: None,
            modified_at: None,
        }
    }

    #[test]
    fn test_layout_list_new() {
        let list = LayoutList::new();
        assert_eq!(list.id(), "layout_list");
        assert!(list.items().is_empty());
    }

    #[test]
    fn test_layout_list_set_layouts() {
        let mut list = LayoutList::new();
        let layouts = vec![
            create_test_layout("设计对照", false),
            create_test_layout("下载整理", true),
        ];

        list.set_layouts(layouts, Some(0));
        assert_eq!(list.items().len(), 2);
        assert!(list.items()[0].is_active);
        assert!(!list.items()[1].is_active);
    }

    #[test]
    fn test_layout_list_item_display_name() {
        let item = LayoutListItem::new(create_test_layout("测试", false), 0, true);
        assert_eq!(item.display_name(), "测试");

        let unnamed = LayoutListItem::new(create_test_layout("", false), 0, true);
        assert_eq!(unnamed.display_name(), "(当前布局)");
    }

    #[test]
    fn test_layout_list_item_properties() {
        let locked = LayoutListItem::new(create_test_layout("锁定", true), 0, false);
        assert!(locked.is_locked());
        assert!(!locked.is_unnamed());

        let unnamed = LayoutListItem::new(create_test_layout("", false), 0, false);
        assert!(!unnamed.is_locked());
        assert!(unnamed.is_unnamed());
    }

    #[test]
    fn test_layout_list_update_hover() {
        let mut list = LayoutList::new();
        list.bounds = Rect::new(0.0, 0.0, 200.0, 100.0);

        let layouts = vec![create_test_layout("测试", false)];
        list.set_layouts(layouts, None);

        // hover 在布局行上
        let result = list.update_hover(10.0, 10.0);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 0);

        // hover 在布局列表外
        let result = list.update_hover(250.0, 10.0);
        assert!(result.is_none());
    }

    #[test]
    fn test_layout_list_button_bounds() {
        let mut list = LayoutList::new();
        list.bounds = Rect::new(0.0, 0.0, 200.0, 100.0);

        let item = LayoutListItem::new(create_test_layout("测试", false), 0, false);
        let lock_bounds = list.lock_button_bounds(&item);
        let reload_bounds = list.reload_button_bounds(&item);
        let menu_bounds = list.menu_button_bounds(&item);

        // 重新加载按钮在锁定按钮左边
        assert!(reload_bounds.x < lock_bounds.x);
        // 锁定按钮在更多菜单按钮左边
        assert!(lock_bounds.x < menu_bounds.x);
    }

    #[test]
    fn test_layout_list_default() {
        let list = LayoutList::default();
        assert!(list.visible);
        assert!(list.items.is_empty());
    }
}

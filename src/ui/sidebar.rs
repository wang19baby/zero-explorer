use super::components::{Component, ComponentState, Rect};
use super::layout_list::LayoutList;
use crate::core::state::LayoutState;

#[derive(Debug, Clone, PartialEq)]
pub enum SidebarPosition {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SidebarItem {
    ThisPC,
    Tags,
    RecentFiles,
    CustomFolder(String),
}

/// 侧栏分组
#[derive(Debug, Clone, PartialEq)]
pub enum SidebarGroup {
    Layouts,
    Recents,
    Bookmarks,
    Tags,
    Storage,
    Places,
}

/// 侧栏分组项
#[derive(Debug, Clone)]
pub struct SidebarGroupItem {
    pub group: SidebarGroup,
    pub name: String,
    pub expanded: bool,
}

impl SidebarGroupItem {
    pub fn new(group: SidebarGroup, name: &str) -> Self {
        Self {
            group,
            name: name.to_string(),
            expanded: true,
        }
    }

    pub fn toggle(&mut self) {
        self.expanded = !self.expanded;
    }
}

#[derive(Debug)]
pub struct Sidebar {
    id: String,
    bounds: Rect,
    state: ComponentState,
    visible: bool,
    position: SidebarPosition,
    width: f32,
    min_width: f32,
    max_width: f32,
    items: Vec<SidebarItem>,
    selected_index: Option<usize>,
    dragging: bool,
    drag_start_x: f32,
    drag_start_width: f32,
    /// 布局列表
    layout_list: LayoutList,
    /// 侧栏分组
    groups: Vec<SidebarGroupItem>,
    /// 当前 hover 的分组标题索引
    hovered_group: Option<usize>,
    /// 布局分组是否展开
    layouts_expanded: bool,
}

impl Sidebar {
    pub fn new() -> Self {
        let groups = vec![
            SidebarGroupItem::new(SidebarGroup::Layouts, "布局"),
            SidebarGroupItem::new(SidebarGroup::Recents, "最近"),
            SidebarGroupItem::new(SidebarGroup::Bookmarks, "书签"),
            SidebarGroupItem::new(SidebarGroup::Tags, "标签"),
            SidebarGroupItem::new(SidebarGroup::Storage, "存储"),
            SidebarGroupItem::new(SidebarGroup::Places, "位置"),
        ];

        Self {
            id: "sidebar".to_string(),
            bounds: Rect::default(),
            state: ComponentState::Normal,
            visible: true,
            position: SidebarPosition::Left,
            width: 200.0,
            min_width: 150.0,
            max_width: 400.0,
            items: vec![
                SidebarItem::ThisPC,
                SidebarItem::Tags,
                SidebarItem::RecentFiles,
            ],
            selected_index: None,
            dragging: false,
            drag_start_x: 0.0,
            drag_start_width: 0.0,
            layout_list: LayoutList::new(),
            groups,
            hovered_group: None,
            layouts_expanded: true,
        }
    }

    pub fn position(&self) -> &SidebarPosition {
        &self.position
    }

    pub fn set_position(&mut self, position: SidebarPosition) {
        self.position = position;
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn set_width(&mut self, width: f32) {
        self.width = width.clamp(self.min_width, self.max_width);
    }

    pub fn items(&self) -> &[SidebarItem] {
        &self.items
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    pub fn select(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected_index = Some(index);
        }
    }

    pub fn start_drag(&mut self, x: f32) {
        self.dragging = true;
        self.drag_start_x = x;
        self.drag_start_width = self.width;
    }

    pub fn update_drag(&mut self, x: f32) {
        if self.dragging {
            let delta = x - self.drag_start_x;
            let new_width = match self.position {
                SidebarPosition::Left => self.drag_start_width + delta,
                SidebarPosition::Right => self.drag_start_width - delta,
            };
            self.width = new_width.clamp(self.min_width, self.max_width);
        }
    }

    pub fn end_drag(&mut self) {
        self.dragging = false;
    }

    pub fn is_dragging(&self) -> bool {
        self.dragging
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// 获取布局列表
    pub fn layout_list(&self) -> &LayoutList {
        &self.layout_list
    }

    /// 获取布局列表（可变）
    pub fn layout_list_mut(&mut self) -> &mut LayoutList {
        &mut self.layout_list
    }

    /// 设置布局
    pub fn set_layouts(&mut self, layouts: Vec<LayoutState>, active_index: Option<usize>) {
        self.layout_list.set_layouts(layouts, active_index);
    }

    /// 获取分组列表
    pub fn groups(&self) -> &[SidebarGroupItem] {
        &self.groups
    }

    /// 切换分组展开状态
    pub fn toggle_group(&mut self, index: usize) {
        if index < self.groups.len() {
            self.groups[index].toggle();
        }
    }

    /// 获取布局分组的边界
    pub fn layouts_group_bounds(&self) -> Option<Rect> {
        let group_height = 28.0;
        let x = self.bounds.x;
        let y = self.bounds.y;

        Some(Rect::new(x, y, self.width, group_height))
    }

    /// 获取布局列表的边界
    pub fn layouts_list_bounds(&self) -> Rect {
        let group_height = 28.0;
        let x = self.bounds.x;
        let y = self.bounds.y + group_height;

        Rect::new(x, y, self.width, self.bounds.height - group_height)
    }

    /// 检查点击是否在布局分组标题上
    pub fn hit_test_layouts_group(&self, x: f32, y: f32) -> bool {
        if let Some(bounds) = self.layouts_group_bounds() {
            bounds.contains(x, y)
        } else {
            false
        }
    }

    /// 检查点击是否在布局分组的 ＋ 按钮上
    pub fn hit_test_add_layout_button(&self, x: f32, y: f32) -> bool {
        if let Some(bounds) = self.layouts_group_bounds() {
            let add_button_x = bounds.x + bounds.width - 24.0;
            let add_button_y = bounds.y + 4.0;
            let add_button_bounds = Rect::new(add_button_x, add_button_y, 20.0, 20.0);
            add_button_bounds.contains(x, y)
        } else {
            false
        }
    }

    /// 获取分组标题的边界
    pub fn group_bounds(&self, index: usize) -> Option<Rect> {
        if index >= self.groups.len() {
            return None;
        }

        let group_height = 28.0;
        let x = self.bounds.x;
        let y = self.bounds.y + (index as f32) * group_height;

        Some(Rect::new(x, y, self.width, group_height))
    }
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Sidebar {
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

    fn render(&self) {}

    fn handle_mouse_move(&mut self, x: f32, y: f32) -> bool {
        if self.dragging {
            self.update_drag(x);
            return true;
        }

        if self.bounds.contains(x, y) {
            self.set_state(ComponentState::Hovered);

            // 检查布局列表 hover
            let layout_list_bounds = self.layouts_list_bounds();
            if layout_list_bounds.contains(x, y) {
                self.layout_list.update_hover(x, y);
            }

            true
        } else {
            if *self.state() == ComponentState::Hovered {
                self.set_state(ComponentState::Normal);
                self.layout_list.update_hover(x, y);
            }
            false
        }
    }

    fn handle_mouse_button_down(&mut self, x: f32, y: f32) -> bool {
        if self.bounds.contains(x, y) {
            self.set_state(ComponentState::Pressed);

            let edge_threshold = 5.0;
            let is_near_edge = match self.position {
                SidebarPosition::Left => (x - self.bounds.x - self.bounds.width).abs() < edge_threshold,
                SidebarPosition::Right => (x - self.bounds.x).abs() < edge_threshold,
            };

            if is_near_edge {
                self.start_drag(x);
                return true;
            }

            // 检查布局分组标题点击
            if self.hit_test_layouts_group(x, y) {
                self.layouts_expanded = !self.layouts_expanded;
                return true;
            }

            // 检查 ＋ 按钮点击
            if self.hit_test_add_layout_button(x, y) {
                return true; // 返回 true 表示点击了 ＋ 按钮
            }

            // 检查布局列表点击
            let layout_list_bounds = self.layouts_list_bounds();
            if layout_list_bounds.contains(x, y) {
                return self.layout_list.handle_mouse_button_down(x, y);
            }

            // 检查其他分组点击
            for (i, _group) in self.groups.iter().enumerate() {
                if let Some(bounds) = self.group_bounds(i) {
                    if bounds.contains(x, y) {
                        self.toggle_group(i);
                        return true;
                    }
                }
            }

            // 检查旧的 items 点击
            let item_height = 32.0;
            let item_index = ((y - self.bounds.y) / item_height) as usize;
            if item_index < self.items.len() {
                self.select(item_index);
            }

            true
        } else {
            false
        }
    }

    fn handle_mouse_button_up(&mut self, _x: f32, _y: f32) -> bool {
        if self.dragging {
            self.end_drag();
            return true;
        }

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
                self.visible = false;
                true
            }
            _ => false,
        }
    }
}

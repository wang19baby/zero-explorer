use crate::ui::components::{Component, ComponentState, Rect};

const TAB_HEIGHT: f32 = 32.0;
const TAB_MIN_WIDTH: f32 = 120.0;
const TAB_MAX_WIDTH: f32 = 200.0;
const CLOSE_BUTTON_SIZE: f32 = 16.0;
const CLOSE_BUTTON_PADDING: f32 = 4.0;

#[derive(Debug, Clone)]
pub struct Tab {
    pub id: usize,
    pub title: String,
    pub bounds: Rect,
    pub visible: bool,
    pub pinned: bool,
    pub dirty: bool,
}

impl Tab {
    pub fn new(id: usize, title: &str, bounds: Rect) -> Self {
        Self {
            id,
            title: title.to_string(),
            bounds,
            visible: true,
            pinned: false,
            dirty: false,
        }
    }

    pub fn close_button_bounds(&self) -> Rect {
        if self.pinned {
            return Rect::new(0.0, 0.0, 0.0, 0.0);
        }

        let x = self.bounds.x + self.bounds.width - CLOSE_BUTTON_SIZE - CLOSE_BUTTON_PADDING;
        let y = self.bounds.y + (self.bounds.height - CLOSE_BUTTON_SIZE) / 2.0;
        Rect::new(x, y, CLOSE_BUTTON_SIZE, CLOSE_BUTTON_SIZE)
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        self.visible && self.bounds.contains(x, y)
    }

    pub fn close_button_hit_test(&self, x: f32, y: f32) -> bool {
        !self.pinned && self.close_button_bounds().contains(x, y)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TabDragState {
    None,
    Dragging { index: usize, start_x: f32 },
}

#[derive(Debug)]
pub struct TabBar {
    id: String,
    bounds: Rect,
    state: ComponentState,
    visible: bool,
    tabs: Vec<Tab>,
    active_tab_index: usize,
    drag_state: TabDragState,
    next_tab_id: usize,
    scroll_offset: f32,
}

impl TabBar {
    pub fn new(id: &str, bounds: Rect) -> Self {
        let mut tabs = Vec::new();
        tabs.push(Tab::new(0, "Tab 1", Rect::new(bounds.x, bounds.y, 150.0, TAB_HEIGHT)));

        Self {
            id: id.to_string(),
            bounds,
            state: ComponentState::Normal,
            visible: true,
            tabs,
            active_tab_index: 0,
            drag_state: TabDragState::None,
            next_tab_id: 1,
            scroll_offset: 0.0,
        }
    }

    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    pub fn tabs_mut(&mut self) -> &mut Vec<Tab> {
        &mut self.tabs
    }

    pub fn active_tab_index(&self) -> usize {
        self.active_tab_index
    }

    pub fn active_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active_tab_index)
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.active_tab_index)
    }

    pub fn set_active_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_tab_index = index;
        }
    }

    pub fn add_tab(&mut self, title: &str) -> usize {
        let id = self.next_tab_id;
        self.next_tab_id += 1;

        let last_tab_x = self.tabs.last().map(|t| t.bounds.x + t.bounds.width).unwrap_or(self.bounds.x);
        let tab_width = (self.bounds.width - self.scroll_offset - self.tabs.len() as f32 * 1.0).max(TAB_MIN_WIDTH) / (self.tabs.len() + 1) as f32;
        let tab_width = tab_width.clamp(TAB_MIN_WIDTH, TAB_MAX_WIDTH);

        let tab = Tab::new(
            id,
            title,
            Rect::new(last_tab_x + 1.0, self.bounds.y, tab_width, TAB_HEIGHT),
        );

        self.tabs.push(tab);
        self.redistribute_tabs();
        self.active_tab_index = self.tabs.len() - 1;
        id
    }

    pub fn close_tab(&mut self, index: usize) -> bool {
        if self.tabs.len() <= 1 {
            return false;
        }

        if index >= self.tabs.len() {
            return false;
        }

        if self.tabs[index].pinned {
            return false;
        }

        self.tabs.remove(index);

        if self.active_tab_index >= self.tabs.len() {
            self.active_tab_index = self.tabs.len() - 1;
        } else if self.active_tab_index > index {
            self.active_tab_index -= 1;
        }

        self.redistribute_tabs();
        true
    }

    pub fn close_active_tab(&mut self) -> bool {
        self.close_tab(self.active_tab_index)
    }

    pub fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab_index = (self.active_tab_index + 1) % self.tabs.len();
        }
    }

    pub fn prev_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab_index = if self.active_tab_index == 0 {
                self.tabs.len() - 1
            } else {
                self.active_tab_index - 1
            };
        }
    }

    pub fn tab_at(&self, x: f32, y: f32) -> Option<usize> {
        if !self.bounds.contains(x, y) {
            return None;
        }

        for (i, tab) in self.tabs.iter().enumerate() {
            if tab.contains(x, y) {
                return Some(i);
            }
        }

        None
    }

    pub fn tab_id_at(&self, x: f32, y: f32) -> Option<usize> {
        self.tab_at(x, y).map(|i| self.tabs[i].id)
    }

    pub fn redistribute_tabs(&mut self) {
        if self.tabs.is_empty() {
            return;
        }

        let available_width = self.bounds.width - self.scroll_offset;
        let tab_width = (available_width / self.tabs.len() as f32).clamp(TAB_MIN_WIDTH, TAB_MAX_WIDTH);
        let mut x = self.bounds.x - self.scroll_offset;

        for tab in &mut self.tabs {
            tab.bounds.x = x;
            tab.bounds.y = self.bounds.y;
            tab.bounds.width = tab_width;
            tab.bounds.height = TAB_HEIGHT;
            x += tab_width + 1.0;
        }
    }

    pub fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
        self.redistribute_tabs();
    }

    pub fn start_tab_drag(&mut self, index: usize, x: f32) {
        self.drag_state = TabDragState::Dragging {
            index,
            start_x: x,
        };
    }

    pub fn update_tab_drag(&mut self, x: f32) {
        if let TabDragState::Dragging { index, start_x } = &self.drag_state {
            let index = *index;
            let delta = x - *start_x;

            if index < self.tabs.len() {
                self.tabs[index].bounds.x += delta;
                let _ = index;
            }
        }
    }

    pub fn end_tab_drag(&mut self) {
        if let TabDragState::Dragging { index, start_x } = self.drag_state {
            let end_x = self.tabs.get(index).map(|t| t.bounds.x).unwrap_or(start_x);
            let _ = end_x;
            self.drag_state = TabDragState::None;
            self.redistribute_tabs();
        } else {
            self.drag_state = TabDragState::None;
        }
    }

    pub fn is_dragging(&self) -> bool {
        self.drag_state != TabDragState::None
    }

    pub fn scroll_left(&mut self) {
        self.scroll_offset = (self.scroll_offset - 50.0).max(0.0);
        self.redistribute_tabs();
    }

    pub fn scroll_right(&mut self) {
        let max_scroll = (self.tabs.len() as f32 * TAB_MIN_WIDTH).max(self.bounds.width);
        self.scroll_offset = (self.scroll_offset + 50.0).min(max_scroll);
        self.redistribute_tabs();
    }
}

impl Component for TabBar {
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
        if self.is_dragging() {
            self.update_tab_drag(x);
            return true;
        }

        if self.bounds.contains(x, y) {
            self.set_state(ComponentState::Hovered);
            return true;
        }

        self.set_state(ComponentState::Normal);
        false
    }

    fn handle_mouse_button_down(&mut self, x: f32, y: f32) -> bool {
        if let Some(index) = self.tab_at(x, y) {
            self.active_tab_index = index;
            self.start_tab_drag(index, x);
            self.set_state(ComponentState::Pressed);
            return true;
        }

        false
    }

    fn handle_mouse_button_up(&mut self, x: f32, y: f32) -> bool {
        if self.is_dragging() {
            self.end_tab_drag();
            self.set_state(ComponentState::Normal);
            return true;
        }

        false
    }
}

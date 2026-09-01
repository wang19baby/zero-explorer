use super::components::{Component, ComponentState, Rect};

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

#[derive(Debug, Clone)]
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
}

impl Sidebar {
    pub fn new() -> Self {
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

            let edge_threshold = 5.0;
            let is_near_edge = match self.position {
                SidebarPosition::Left => (x - self.bounds.x - self.bounds.width).abs() < edge_threshold,
                SidebarPosition::Right => (x - self.bounds.x).abs() < edge_threshold,
            };

            if is_near_edge {
                self.start_drag(x);
                return true;
            }

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

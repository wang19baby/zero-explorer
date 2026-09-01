use super::components::{Component, ComponentState, Rect};

#[derive(Debug, Clone, PartialEq)]
pub enum StatusBarLayout {
    SinglePanel,
    DualPanel,
    TriplePanel,
    QuadPanel,
}

#[derive(Debug, Clone)]
pub struct StatusBar {
    id: String,
    bounds: Rect,
    state: ComponentState,
    visible: bool,
    panel_count: usize,
    selected_count: usize,
    current_path: String,
    layout: StatusBarLayout,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            id: "status_bar".to_string(),
            bounds: Rect::default(),
            state: ComponentState::Normal,
            visible: true,
            panel_count: 1,
            selected_count: 0,
            current_path: String::new(),
            layout: StatusBarLayout::SinglePanel,
        }
    }

    pub fn set_panel_count(&mut self, count: usize) {
        self.panel_count = count;
    }

    pub fn set_selected_count(&mut self, count: usize) {
        self.selected_count = count;
    }

    pub fn set_current_path(&mut self, path: &str) {
        self.current_path = path.to_string();
    }

    pub fn layout(&self) -> &StatusBarLayout {
        &self.layout
    }

    pub fn set_layout(&mut self, layout: StatusBarLayout) {
        self.layout = layout;
    }

    pub fn cycle_layout(&mut self) {
        self.layout = match self.layout {
            StatusBarLayout::SinglePanel => StatusBarLayout::DualPanel,
            StatusBarLayout::DualPanel => StatusBarLayout::TriplePanel,
            StatusBarLayout::TriplePanel => StatusBarLayout::QuadPanel,
            StatusBarLayout::QuadPanel => StatusBarLayout::SinglePanel,
        };
    }

    pub fn get_status_text(&self) -> String {
        let panel_info = format!("{} panel(s)", self.panel_count);
        let selected_info = if self.selected_count > 0 {
            format!(", {} selected", self.selected_count)
        } else {
            String::new()
        };
        format!("{}{} | {}", panel_info, selected_info, self.current_path)
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for StatusBar {
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

    fn handle_mouse_button_down(&mut self, x: f32, y: f32) -> bool {
        if self.bounds.contains(x, y) {
            self.set_state(ComponentState::Pressed);
            true
        } else {
            false
        }
    }

    fn handle_mouse_button_up(&mut self, x: f32, y: f32) -> bool {
        if *self.state() == ComponentState::Pressed {
            self.set_state(ComponentState::Hovered);

            let layout_icon_x = self.bounds.x + self.bounds.width - 32.0;
            if x >= layout_icon_x {
                self.cycle_layout();
                return true;
            }

            true
        } else {
            false
        }
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

    fn handle_key_down(&mut self, key: u32) -> bool {
        match key {
            116 => {
                self.cycle_layout();
                true
            }
            _ => false,
        }
    }
}

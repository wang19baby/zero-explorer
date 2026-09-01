use crate::ui::components::{Component, ComponentState, Rect};

const SEGMENT_PADDING: f32 = 8.0;
const SEPARATOR_WIDTH: f32 = 16.0;
const INPUT_HEIGHT: f32 = 24.0;

#[derive(Debug, Clone)]
pub struct BreadcrumbSegment {
    pub name: String,
    pub path: String,
    pub bounds: Rect,
}

impl BreadcrumbSegment {
    pub fn new(name: &str, path: &str, bounds: Rect) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_string(),
            bounds,
        }
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        self.bounds.contains(x, y)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BreadcrumbMode {
    Breadcrumb,
    Input,
}

#[derive(Debug)]
pub struct Breadcrumb {
    id: String,
    bounds: Rect,
    state: ComponentState,
    visible: bool,
    segments: Vec<BreadcrumbSegment>,
    current_path: String,
    mode: BreadcrumbMode,
    input_text: String,
    input_cursor: usize,
    input_focused: bool,
}

impl Breadcrumb {
    pub fn new(id: &str, bounds: Rect) -> Self {
        let mut breadcrumb = Self {
            id: id.to_string(),
            bounds,
            state: ComponentState::Normal,
            visible: true,
            segments: Vec::new(),
            current_path: String::new(),
            mode: BreadcrumbMode::Breadcrumb,
            input_text: String::new(),
            input_cursor: 0,
            input_focused: false,
        };
        breadcrumb.set_path("C:\\");
        breadcrumb
    }

    pub fn set_path(&mut self, path: &str) {
        self.current_path = path.to_string();
        self.input_text = path.to_string();
        self.input_cursor = self.input_text.len();
        self.rebuild_segments();
    }

    pub fn path(&self) -> &str {
        &self.current_path
    }

    pub fn input_text(&self) -> &str {
        &self.input_text
    }

    pub fn mode(&self) -> &BreadcrumbMode {
        &self.mode
    }

    fn rebuild_segments(&mut self) {
        self.segments.clear();
        let parts: Vec<&str> = self.current_path.split('\\').filter(|s| !s.is_empty()).collect();
        let mut x = self.bounds.x;

        for (i, part) in parts.iter().enumerate() {
            let name = if i == 0 {
                format!("{}\\", part)
            } else {
                part.to_string()
            };

            let width = (name.len() as f32 * 8.0 + SEGMENT_PADDING * 2.0).min(150.0);
            let segment = BreadcrumbSegment::new(
                &name,
                &self.current_path[..self.current_path.find(part).unwrap_or(0) + part.len()],
                Rect::new(x, self.bounds.y, width, self.bounds.height),
            );

            x += width + SEPARATOR_WIDTH;
            self.segments.push(segment);
        }
    }

    pub fn segment_at(&self, x: f32, y: f32) -> Option<usize> {
        if !self.bounds.contains(x, y) {
            return None;
        }

        for (i, segment) in self.segments.iter().enumerate() {
            if segment.contains(x, y) {
                return Some(i);
            }
        }

        None
    }

    pub fn switch_to_input(&mut self) {
        self.mode = BreadcrumbMode::Input;
        self.input_text = self.current_path.clone();
        self.input_cursor = self.input_text.len();
        self.input_focused = true;
    }

    pub fn switch_to_breadcrumb(&mut self) {
        self.mode = BreadcrumbMode::Breadcrumb;
        self.input_focused = false;
    }

    pub fn input_accept(&mut self) -> String {
        let path = self.input_text.clone();
        self.current_path = path.clone();
        self.switch_to_breadcrumb();
        self.rebuild_segments();
        path
    }

    pub fn input_cancel(&mut self) {
        self.input_text = self.current_path.clone();
        self.switch_to_breadcrumb();
    }

    pub fn set_input_text(&mut self, text: &str) {
        self.input_text = text.to_string();
        self.input_cursor = self.input_text.len();
    }

    pub fn input_cursor_position(&self) -> usize {
        self.input_cursor
    }

    pub fn set_input_cursor_position(&mut self, pos: usize) {
        self.input_cursor = pos.min(self.input_text.len());
    }

    pub fn segments(&self) -> &[BreadcrumbSegment] {
        &self.segments
    }
}

impl Component for Breadcrumb {
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
            return true;
        }

        self.set_state(ComponentState::Normal);
        false
    }

    fn handle_mouse_button_down(&mut self, x: f32, y: f32) -> bool {
        if self.bounds.contains(x, y) {
            self.set_state(ComponentState::Pressed);
            return true;
        }

        false
    }

    fn handle_mouse_button_up(&mut self, x: f32, y: f32) -> bool {
        if *self.state() == ComponentState::Pressed {
            self.set_state(ComponentState::Hovered);
            return true;
        }

        false
    }

    fn handle_key_down(&mut self, key: u32) -> bool {
        match self.mode {
            BreadcrumbMode::Input => match key {
                13 => {
                    self.input_accept();
                    true
                }
                27 => {
                    self.input_cancel();
                    true
                }
                37 => {
                    if self.input_cursor > 0 {
                        self.input_cursor -= 1;
                    }
                    true
                }
                39 => {
                    if self.input_cursor < self.input_text.len() {
                        self.input_cursor += 1;
                    }
                    true
                }
                36 => {
                    self.input_cursor = 0;
                    true
                }
                35 => {
                    self.input_cursor = self.input_text.len();
                    true
                }
                8 => {
                    if self.input_cursor > 0 {
                        self.input_cursor -= 1;
                        self.input_text.remove(self.input_cursor);
                    }
                    true
                }
                46 => {
                    if self.input_cursor < self.input_text.len() {
                        self.input_text.remove(self.input_cursor);
                    }
                    true
                }
                _ => false,
            },
            BreadcrumbMode::Breadcrumb => {
                if key == 13 {
                    self.switch_to_input();
                    true
                } else {
                    false
                }
            }
        }
    }

    fn handle_char_input(&mut self, ch: char) -> bool {
        if self.mode == BreadcrumbMode::Input && self.input_focused {
            self.input_text.insert(self.input_cursor, ch);
            self.input_cursor += 1;
            true
        } else {
            false
        }
    }
}

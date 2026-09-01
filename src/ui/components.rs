#[derive(Debug, Clone)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }

    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = (self.x + self.width).min(other.x + other.width);
        let bottom = (self.y + self.height).min(other.y + other.height);

        if right > x && bottom > y {
            Some(Rect::new(x, y, right - x, bottom - y))
        } else {
            None
        }
    }

    pub fn expand(&self, padding: f32) -> Rect {
        Rect::new(
            self.x - padding,
            self.y - padding,
            self.width + padding * 2.0,
            self.height + padding * 2.0,
        )
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentState {
    Normal,
    Hovered,
    Pressed,
    Focused,
    Disabled,
}

pub trait Component {
    fn id(&self) -> &str;
    fn bounds(&self) -> &Rect;
    fn bounds_mut(&mut self) -> &mut Rect;
    fn state(&self) -> &ComponentState;
    fn set_state(&mut self, state: ComponentState);

    fn update(&mut self, _dt: f32) {}
    fn render(&self) {}

    fn is_visible(&self) -> bool {
        true
    }

    fn set_visible(&mut self, _visible: bool) {}

    fn handle_mouse_move(&mut self, x: f32, y: f32) -> bool {
        if self.bounds().contains(x, y) {
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
        if self.bounds().contains(x, y) {
            self.set_state(ComponentState::Pressed);
            true
        } else {
            false
        }
    }

    fn handle_mouse_button_up(&mut self, x: f32, y: f32) -> bool {
        if *self.state() == ComponentState::Pressed {
            self.set_state(ComponentState::Hovered);
            true
        } else {
            false
        }
    }

    fn handle_key_down(&mut self, _key: u32) -> bool {
        false
    }

    fn handle_key_up(&mut self, _key: u32) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct Button {
    id: String,
    bounds: Rect,
    state: ComponentState,
    text: String,
    visible: bool,
}

impl Button {
    pub fn new(id: &str, text: &str, bounds: Rect) -> Self {
        Self {
            id: id.to_string(),
            bounds,
            state: ComponentState::Normal,
            text: text.to_string(),
            visible: true,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }
}

impl Component for Button {
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
}

#[derive(Debug)]
pub struct Panel {
    id: String,
    bounds: Rect,
    state: ComponentState,
    visible: bool,
    background_color: Option<[f32; 4]>,
}

impl Panel {
    pub fn new(id: &str, bounds: Rect) -> Self {
        Self {
            id: id.to_string(),
            bounds,
            state: ComponentState::Normal,
            visible: true,
            background_color: None,
        }
    }

    pub fn set_background_color(&mut self, color: [f32; 4]) {
        self.background_color = Some(color);
    }

    pub fn background_color(&self) -> Option<[f32; 4]> {
        self.background_color
    }
}

impl Component for Panel {
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
}

#[derive(Debug)]
pub struct Label {
    id: String,
    bounds: Rect,
    state: ComponentState,
    text: String,
    visible: bool,
}

impl Label {
    pub fn new(id: &str, text: &str, bounds: Rect) -> Self {
        Self {
            id: id.to_string(),
            bounds,
            state: ComponentState::Normal,
            text: text.to_string(),
            visible: true,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }
}

impl Component for Label {
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
}

#[derive(Debug)]
pub struct TextInput {
    id: String,
    bounds: Rect,
    state: ComponentState,
    text: String,
    placeholder: String,
    visible: bool,
    cursor_position: usize,
    focused: bool,
}

impl TextInput {
    pub fn new(id: &str, bounds: Rect) -> Self {
        Self {
            id: id.to_string(),
            bounds,
            state: ComponentState::Normal,
            text: String::new(),
            placeholder: String::new(),
            visible: true,
            cursor_position: 0,
            focused: false,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.cursor_position = self.text.len();
    }

    pub fn placeholder(&self) -> &str {
        &self.placeholder
    }

    pub fn set_placeholder(&mut self, placeholder: &str) {
        self.placeholder = placeholder.to_string();
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
}

impl Component for TextInput {
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

    fn handle_mouse_button_down(&mut self, x: f32, y: f32) -> bool {
        if self.bounds().contains(x, y) {
            self.focused = true;
            self.set_state(ComponentState::Focused);
            true
        } else {
            self.focused = false;
            self.set_state(ComponentState::Normal);
            false
        }
    }

    fn handle_key_down(&mut self, key: u32) -> bool {
        if !self.focused {
            return false;
        }

        match key {
            // Backspace
            8 => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.text.remove(self.cursor_position);
                }
                true
            }
            // Delete
            46 => {
                if self.cursor_position < self.text.len() {
                    self.text.remove(self.cursor_position);
                }
                true
            }
            // Left arrow
            37 => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
                true
            }
            // Right arrow
            39 => {
                if self.cursor_position < self.text.len() {
                    self.cursor_position += 1;
                }
                true
            }
            // Home
            36 => {
                self.cursor_position = 0;
                true
            }
            // End
            35 => {
                self.cursor_position = self.text.len();
                true
            }
            _ => false,
        }
    }
}

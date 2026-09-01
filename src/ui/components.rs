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

    fn handle_char_input(&mut self, _ch: char) -> bool {
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

    fn handle_char_input(&mut self, ch: char) -> bool {
        if !self.focused {
            return false;
        }

        if ch == '\r' || ch == '\n' || ch == '\x1b' {
            return false;
        }

        self.text.insert(self.cursor_position, ch);
        self.cursor_position += 1;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_new() {
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(rect.x, 10.0);
        assert_eq!(rect.y, 20.0);
        assert_eq!(rect.width, 100.0);
        assert_eq!(rect.height, 50.0);
    }

    #[test]
    fn test_rect_contains_point_inside() {
        let rect = Rect::new(0.0, 0.0, 100.0, 50.0);
        assert!(rect.contains(50.0, 25.0));
    }

    #[test]
    fn test_rect_contains_point_outside() {
        let rect = Rect::new(0.0, 0.0, 100.0, 50.0);
        assert!(!rect.contains(150.0, 25.0));
    }

    #[test]
    fn test_rect_contains_point_on_edge() {
        let rect = Rect::new(0.0, 0.0, 100.0, 50.0);
        assert!(rect.contains(0.0, 0.0));
        assert!(rect.contains(100.0, 50.0));
    }

    #[test]
    fn test_rect_intersection_overlapping() {
        let r1 = Rect::new(0.0, 0.0, 100.0, 100.0);
        let r2 = Rect::new(50.0, 50.0, 100.0, 100.0);
        let intersection = r1.intersection(&r2).unwrap();
        assert_eq!(intersection.x, 50.0);
        assert_eq!(intersection.y, 50.0);
        assert_eq!(intersection.width, 50.0);
        assert_eq!(intersection.height, 50.0);
    }

    #[test]
    fn test_rect_intersection_no_overlap() {
        let r1 = Rect::new(0.0, 0.0, 50.0, 50.0);
        let r2 = Rect::new(100.0, 100.0, 50.0, 50.0);
        assert!(r1.intersection(&r2).is_none());
    }

    #[test]
    fn test_rect_expand() {
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        let expanded = rect.expand(5.0);
        assert_eq!(expanded.x, 5.0);
        assert_eq!(expanded.y, 15.0);
        assert_eq!(expanded.width, 110.0);
        assert_eq!(expanded.height, 60.0);
    }

    #[test]
    fn test_rect_default() {
        let rect = Rect::default();
        assert_eq!(rect.x, 0.0);
        assert_eq!(rect.y, 0.0);
        assert_eq!(rect.width, 0.0);
        assert_eq!(rect.height, 0.0);
    }

    #[test]
    fn test_button_new() {
        let button = Button::new("test", "Click me", Rect::new(0.0, 0.0, 100.0, 30.0));
        assert_eq!(button.id(), "test");
        assert_eq!(button.text(), "Click me");
        assert_eq!(*button.state(), ComponentState::Normal);
    }

    #[test]
    fn test_button_state_transitions() {
        let mut button = Button::new("test", "Click me", Rect::new(0.0, 0.0, 100.0, 30.0));
        assert_eq!(*button.state(), ComponentState::Normal);
        
        button.handle_mouse_move(50.0, 15.0);
        assert_eq!(*button.state(), ComponentState::Hovered);
        
        button.handle_mouse_button_down(50.0, 15.0);
        assert_eq!(*button.state(), ComponentState::Pressed);
        
        button.handle_mouse_button_up(50.0, 15.0);
        assert_eq!(*button.state(), ComponentState::Hovered);
    }

    #[test]
    fn test_button_set_text() {
        let mut button = Button::new("test", "Original", Rect::new(0.0, 0.0, 100.0, 30.0));
        button.set_text("Updated");
        assert_eq!(button.text(), "Updated");
    }

    #[test]
    fn test_panel_new() {
        let panel = Panel::new("panel1", Rect::new(0.0, 0.0, 200.0, 300.0));
        assert_eq!(panel.id(), "panel1");
    }

    #[test]
    fn test_text_input_new() {
        let input = TextInput::new("input1", Rect::new(0.0, 0.0, 200.0, 30.0));
        assert_eq!(input.id(), "input1");
        assert_eq!(input.text(), "");
    }

    #[test]
    fn test_text_input_set_text() {
        let mut input = TextInput::new("input1", Rect::new(0.0, 0.0, 200.0, 30.0));
        input.set_text("hello");
        assert_eq!(input.text(), "hello");
    }

    #[test]
    fn test_text_input_char_input() {
        let mut input = TextInput::new("input1", Rect::new(0.0, 0.0, 200.0, 30.0));
        input.set_focused(true);
        input.handle_char_input('h');
        input.handle_char_input('i');
        assert_eq!(input.text(), "hi");
    }

    #[test]
    fn test_text_input_backspace() {
        let mut input = TextInput::new("input1", Rect::new(0.0, 0.0, 200.0, 30.0));
        input.set_text("hello");
        input.set_focused(true);
        input.handle_key_down(8); // Backspace
        assert_eq!(input.text(), "hell");
    }

    #[test]
    fn test_text_input_delete() {
        let mut input = TextInput::new("input1", Rect::new(0.0, 0.0, 200.0, 30.0));
        input.set_text("hello");
        input.set_focused(true);
        input.cursor_position = 0;
        input.handle_key_down(46); // Delete at pos 0 removes 'h'
        assert_eq!(input.text(), "ello");
    }

    #[test]
    fn test_text_input_cursor_navigation() {
        let mut input = TextInput::new("input1", Rect::new(0.0, 0.0, 200.0, 30.0));
        input.set_text("hello");
        input.set_focused(true);
        
        input.handle_key_down(36); // Home
        assert_eq!(input.cursor_position, 0);
        
        input.handle_key_down(35); // End
        assert_eq!(input.cursor_position, 5);
    }

    #[test]
    fn test_text_input_placeholder() {
        let mut input = TextInput::new("input1", Rect::new(0.0, 0.0, 200.0, 30.0));
        input.set_placeholder("Enter text...");
        assert_eq!(input.placeholder(), "Enter text...");
    }

    #[test]
    fn test_text_input_focus() {
        let mut input = TextInput::new("input1", Rect::new(0.0, 0.0, 200.0, 30.0));
        assert!(!input.is_focused());
        
        input.set_focused(true);
        assert!(input.is_focused());
    }
}

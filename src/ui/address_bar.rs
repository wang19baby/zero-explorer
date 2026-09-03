use std::path::PathBuf;

use super::components::{Component, ComponentState, Rect};

#[derive(Debug, Clone, PartialEq)]
pub enum AddressBarMode {
    Breadcrumb,
    Input,
}

#[derive(Debug, Clone)]
pub struct AddressBar {
    id: String,
    bounds: Rect,
    state: ComponentState,
    visible: bool,
    mode: AddressBarMode,
    current_path: PathBuf,
    input_value: String,
    cursor_position: usize,
    history: Vec<PathBuf>,
    history_index: Option<usize>,
}

impl AddressBar {
    pub fn new() -> Self {
        Self {
            id: "address_bar".to_string(),
            bounds: Rect::default(),
            state: ComponentState::Normal,
            visible: true,
            mode: AddressBarMode::Breadcrumb,
            current_path: PathBuf::new(),
            input_value: String::new(),
            cursor_position: 0,
            history: Vec::new(),
            history_index: None,
        }
    }

    pub fn set_path(&mut self, path: PathBuf) {
        self.current_path = path.clone();
        self.input_value = path.display().to_string();
        self.cursor_position = self.input_value.len();

        if self.history.last() != Some(&path) {
            self.history.push(path);
            self.history_index = Some(self.history.len() - 1);
        }

        self.mode = AddressBarMode::Breadcrumb;
    }

    pub fn current_path(&self) -> &PathBuf {
        &self.current_path
    }

    pub fn mode(&self) -> &AddressBarMode {
        &self.mode
    }

    pub fn input_value(&self) -> &str {
        &self.input_value
    }

    pub fn focus(&mut self) {
        self.state = ComponentState::Focused;
        self.mode = AddressBarMode::Input;
        self.input_value = self.current_path.display().to_string();
        self.cursor_position = self.input_value.len();
    }

    pub fn blur(&mut self) {
        self.state = ComponentState::Normal;
        self.mode = AddressBarMode::Breadcrumb;
    }

    pub fn go_back(&mut self) -> Option<PathBuf> {
        if let Some(index) = self.history_index {
            if index > 0 {
                self.history_index = Some(index - 1);
                return self.history.get(index - 1).cloned();
            }
        }
        None
    }

    pub fn go_forward(&mut self) -> Option<PathBuf> {
        if let Some(index) = self.history_index {
            if index + 1 < self.history.len() {
                self.history_index = Some(index + 1);
                return self.history.get(index + 1).cloned();
            }
        }
        None
    }

    pub fn go_up(&self) -> Option<PathBuf> {
        self.current_path.parent().map(|p| p.to_path_buf())
    }

    pub fn can_go_back(&self) -> bool {
        self.history_index.is_some_and(|i| i > 0)
    }

    pub fn can_go_forward(&self) -> bool {
        self.history_index.is_some_and(|i| i + 1 < self.history.len())
    }

    pub fn can_go_up(&self) -> bool {
        self.current_path.parent().is_some()
    }

    fn handle_input_char(&mut self, ch: char) {
        self.input_value.insert(self.cursor_position, ch);
        self.cursor_position += 1;
    }

    fn handle_input_backspace(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.input_value.remove(self.cursor_position);
        }
    }

    fn handle_input_delete(&mut self) {
        if self.cursor_position < self.input_value.len() {
            self.input_value.remove(self.cursor_position);
        }
    }

    fn handle_input_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    fn handle_input_right(&mut self) {
        if self.cursor_position < self.input_value.len() {
            self.cursor_position += 1;
        }
    }

    fn handle_input_home(&mut self) {
        self.cursor_position = 0;
    }

    fn handle_input_end(&mut self) {
        self.cursor_position = self.input_value.len();
    }

    fn get_confirm_path(&self) -> PathBuf {
        PathBuf::from(self.input_value.trim())
    }
}

impl Default for AddressBar {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for AddressBar {
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

    fn handle_key_down(&mut self, key: u32) -> bool {
        match key {
            13 => {
                if self.mode == AddressBarMode::Input {
                    let path = self.get_confirm_path();
                    if path.exists() {
                        self.set_path(path);
                        return true;
                    }
                }
                false
            }
            27 => {
                if self.mode == AddressBarMode::Input {
                    self.blur();
                    return true;
                }
                false
            }
            37 => {
                if self.mode == AddressBarMode::Input {
                    self.handle_input_left();
                    return true;
                }
                false
            }
            39 => {
                if self.mode == AddressBarMode::Input {
                    self.handle_input_right();
                    return true;
                }
                false
            }
            36 => {
                if self.mode == AddressBarMode::Input {
                    self.handle_input_home();
                    return true;
                }
                false
            }
            35 => {
                if self.mode == AddressBarMode::Input {
                    self.handle_input_end();
                    return true;
                }
                false
            }
            8 => {
                if self.mode == AddressBarMode::Input {
                    self.handle_input_backspace();
                    return true;
                }
                false
            }
            46 => {
                if self.mode == AddressBarMode::Input {
                    self.handle_input_delete();
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    fn handle_char_input(&mut self, ch: char) -> bool {
        if self.mode == AddressBarMode::Input {
            if ch == '\r' || ch == '\n' {
                return false;
            }
            if ch == '\x1b' {
                return false;
            }
            self.handle_input_char(ch);
            return true;
        }
        false
    }

    fn handle_mouse_button_down(&mut self, x: f32, y: f32) -> bool {
        if self.bounds.contains(x, y) {
            self.focus();
            true
        } else {
            false
        }
    }
}

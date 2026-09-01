use std::path::PathBuf;

use super::components::{Component, ComponentState, Rect};

#[derive(Debug, Clone, PartialEq)]
pub enum SearchMode {
    FileName,
    Content,
    Regex,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub path: PathBuf,
    pub name: String,
    pub matched_line: Option<usize>,
    pub matched_text: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SearchPanel {
    id: String,
    bounds: Rect,
    state: ComponentState,
    visible: bool,
    query: String,
    cursor_position: usize,
    mode: SearchMode,
    results: Vec<SearchResult>,
    selected_index: Option<usize>,
    is_searching: bool,
    case_sensitive: bool,
    match_whole_word: bool,
}

impl SearchPanel {
    pub fn new() -> Self {
        Self {
            id: "search_panel".to_string(),
            bounds: Rect::default(),
            state: ComponentState::Normal,
            visible: false,
            query: String::new(),
            cursor_position: 0,
            mode: SearchMode::FileName,
            results: Vec::new(),
            selected_index: None,
            is_searching: false,
            case_sensitive: false,
            match_whole_word: false,
        }
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.query.clear();
        self.cursor_position = 0;
        self.results.clear();
        self.selected_index = None;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.query.clear();
        self.results.clear();
        self.selected_index = None;
    }

    pub fn toggle(&mut self) {
        if self.visible {
            self.hide();
        } else {
            self.show();
        }
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn set_query(&mut self, query: &str) {
        self.query = query.to_string();
        self.cursor_position = self.query.len();
    }

    pub fn mode(&self) -> &SearchMode {
        &self.mode
    }

    pub fn set_mode(&mut self, mode: SearchMode) {
        self.mode = mode;
    }

    pub fn results(&self) -> &[SearchResult] {
        &self.results
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    pub fn select(&mut self, index: usize) {
        if index < self.results.len() {
            self.selected_index = Some(index);
        }
    }

    pub fn selected_result(&self) -> Option<&SearchResult> {
        self.selected_index.and_then(|i| self.results.get(i))
    }

    pub fn is_searching(&self) -> bool {
        self.is_searching
    }

    pub fn set_searching(&mut self, searching: bool) {
        self.is_searching = searching;
    }

    pub fn set_results(&mut self, results: Vec<SearchResult>) {
        self.results = results;
        self.selected_index = if self.results.is_empty() {
            None
        } else {
            Some(0)
        };
    }

    pub fn case_sensitive(&self) -> bool {
        self.case_sensitive
    }

    pub fn set_case_sensitive(&mut self, case_sensitive: bool) {
        self.case_sensitive = case_sensitive;
    }

    pub fn match_whole_word(&self) -> bool {
        self.match_whole_word
    }

    pub fn set_match_whole_word(&mut self, match_whole_word: bool) {
        self.match_whole_word = match_whole_word;
    }

    pub fn matches_query(&self, name: &str) -> bool {
        if self.query.is_empty() {
            return true;
        }

        let query = if self.case_sensitive {
            self.query.clone()
        } else {
            self.query.to_lowercase()
        };

        let name = if self.case_sensitive {
            name.to_string()
        } else {
            name.to_lowercase()
        };

        if self.match_whole_word {
            name == query
        } else {
            name.contains(&query)
        }
    }

    pub fn highlight_match(&self, text: &str) -> (String, Vec<usize>) {
        if self.query.is_empty() {
            return (text.to_string(), Vec::new());
        }

        let query = if self.case_sensitive {
            self.query.clone()
        } else {
            self.query.to_lowercase()
        };

        let text_lower = if self.case_sensitive {
            text.to_string()
        } else {
            text.to_lowercase()
        };

        let mut highlights = Vec::new();
        let mut start = 0;

        while let Some(pos) = text_lower[start..].find(&query) {
            let actual_pos = start + pos;
            for i in 0..query.len() {
                highlights.push(actual_pos + i);
            }
            start = actual_pos + 1;
        }

        (text.to_string(), highlights)
    }
}

impl Default for SearchPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for SearchPanel {
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
        if !self.visible {
            return false;
        }

        match key {
            27 => {
                self.hide();
                true
            }
            13 => {
                // Enter - confirm selection
                true
            }
            38 => {
                // Up arrow
                if let Some(index) = self.selected_index {
                    if index > 0 {
                        self.selected_index = Some(index - 1);
                    }
                }
                true
            }
            40 => {
                // Down arrow
                if let Some(index) = self.selected_index {
                    if index + 1 < self.results.len() {
                        self.selected_index = Some(index + 1);
                    }
                } else if !self.results.is_empty() {
                    self.selected_index = Some(0);
                }
                true
            }
            8 => {
                // Backspace
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.query.remove(self.cursor_position);
                }
                true
            }
            46 => {
                // Delete
                if self.cursor_position < self.query.len() {
                    self.query.remove(self.cursor_position);
                }
                true
            }
            37 => {
                // Left arrow
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
                true
            }
            39 => {
                // Right arrow
                if self.cursor_position < self.query.len() {
                    self.cursor_position += 1;
                }
                true
            }
            36 => {
                // Home
                self.cursor_position = 0;
                true
            }
            35 => {
                // End
                self.cursor_position = self.query.len();
                true
            }
            _ => false,
        }
    }

    fn handle_char_input(&mut self, ch: char) -> bool {
        if !self.visible {
            return false;
        }

        if ch == '\r' || ch == '\n' || ch == '\x1b' {
            return false;
        }

        self.query.insert(self.cursor_position, ch);
        self.cursor_position += 1;
        true
    }

    fn handle_mouse_button_down(&mut self, x: f32, y: f32) -> bool {
        if self.visible && self.bounds.contains(x, y) {
            self.set_state(ComponentState::Focused);
            true
        } else {
            false
        }
    }
}

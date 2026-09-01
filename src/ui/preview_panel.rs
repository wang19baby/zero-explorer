use std::path::PathBuf;

use super::components::{Component, ComponentState, Rect};

#[derive(Debug, Clone, PartialEq)]
pub enum PreviewMode {
    Image,
    Text,
    Pdf,
    Archive,
    FileInfo,
    Unsupported,
}

#[derive(Debug, Clone)]
pub struct PreviewPanel {
    id: String,
    bounds: Rect,
    state: ComponentState,
    visible: bool,
    mode: PreviewMode,
    file_path: Option<PathBuf>,
    file_name: String,
    file_size: u64,
    file_type: String,
    content: Option<String>,
    zoom: f32,
    scroll_offset: f32,
}

impl PreviewPanel {
    pub fn new() -> Self {
        Self {
            id: "preview_panel".to_string(),
            bounds: Rect::default(),
            state: ComponentState::Normal,
            visible: false,
            mode: PreviewMode::Unsupported,
            file_path: None,
            file_name: String::new(),
            file_size: 0,
            file_type: String::new(),
            content: None,
            zoom: 1.0,
            scroll_offset: 0.0,
        }
    }

    pub fn show(&mut self, path: &PathBuf) {
        self.file_path = Some(path.clone());
        self.file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        self.file_type = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        
        self.mode = self.detect_mode();
        self.visible = true;
        self.scroll_offset = 0.0;
        self.zoom = 1.0;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.file_path = None;
        self.content = None;
    }

    pub fn toggle(&mut self, path: &PathBuf) {
        if self.visible && self.file_path.as_ref() == Some(path) {
            self.hide();
        } else {
            self.show(path);
        }
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn mode(&self) -> &PreviewMode {
        &self.mode
    }

    pub fn file_path(&self) -> Option<&PathBuf> {
        self.file_path.as_ref()
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn file_size(&self) -> u64 {
        self.file_size
    }

    pub fn file_type(&self) -> &str {
        &self.file_type
    }

    pub fn content(&self) -> Option<&str> {
        self.content.as_deref()
    }

    pub fn set_content(&mut self, content: String) {
        self.content = Some(content);
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.1, 5.0);
    }

    pub fn scroll(&mut self, delta: f32) {
        self.scroll_offset = (self.scroll_offset + delta).max(0.0);
    }

    fn detect_mode(&self) -> PreviewMode {
        match self.file_type.as_str() {
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" => PreviewMode::Image,
            "txt" | "md" | "rs" | "py" | "js" | "ts" | "html" | "css" | "json" | "yaml" | "yml" | "toml" | "xml" => PreviewMode::Text,
            "pdf" => PreviewMode::Pdf,
            "zip" | "rar" | "7z" | "tar" | "gz" => PreviewMode::Archive,
            _ => PreviewMode::Unsupported,
        }
    }

    pub fn format_size(size: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = 1024 * KB;
        const GB: u64 = 1024 * MB;

        if size >= GB {
            format!("{:.1} GB", size as f64 / GB as f64)
        } else if size >= MB {
            format!("{:.1} MB", size as f64 / MB as f64)
        } else if size >= KB {
            format!("{:.1} KB", size as f64 / KB as f64)
        } else {
            format!("{} B", size)
        }
    }
}

impl Default for PreviewPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for PreviewPanel {
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
            27 => {
                self.hide();
                true
            }
            33 => {
                self.scroll(-50.0);
                true
            }
            34 => {
                self.scroll(50.0);
                true
            }
            _ => false,
        }
    }
}

use std::path::PathBuf;

use super::components::{Component, ComponentState, Rect};

#[derive(Debug, Clone)]
pub struct GalleryItem {
    pub path: PathBuf,
    pub name: String,
    pub thumbnail: Option<Vec<u8>>,
    pub width: u32,
    pub height: u32,
    pub file_size: u64,
}

impl GalleryItem {
    pub fn new(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        Self {
            path,
            name,
            thumbnail: None,
            width: 0,
            height: 0,
            file_size: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GalleryMode {
    Grid,
    Slideshow,
}

#[derive(Debug, Clone)]
pub struct GalleryView {
    id: String,
    bounds: Rect,
    state: ComponentState,
    visible: bool,
    items: Vec<GalleryItem>,
    selected_index: Option<usize>,
    mode: GalleryMode,
    zoom: f32,
    thumbnail_size: f32,
    columns: usize,
    scroll_offset: usize,
    slideshow_interval: f32,
    slideshow_timer: f32,
}

impl GalleryView {
    pub fn new() -> Self {
        Self {
            id: "gallery_view".to_string(),
            bounds: Rect::default(),
            state: ComponentState::Normal,
            visible: true,
            items: Vec::new(),
            selected_index: None,
            mode: GalleryMode::Grid,
            zoom: 1.0,
            thumbnail_size: 150.0,
            columns: 4,
            scroll_offset: 0,
            slideshow_interval: 3.0,
            slideshow_timer: 0.0,
        }
    }

    pub fn items(&self) -> &[GalleryItem] {
        &self.items
    }

    pub fn items_mut(&mut self) -> &mut Vec<GalleryItem> {
        &mut self.items
    }

    pub fn set_items(&mut self, items: Vec<GalleryItem>) {
        self.items = items;
        self.selected_index = if self.items.is_empty() {
            None
        } else {
            Some(0)
        };
        self.scroll_offset = 0;
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    pub fn select(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected_index = Some(index);
        }
    }

    pub fn selected_item(&self) -> Option<&GalleryItem> {
        self.selected_index.and_then(|i| self.items.get(i))
    }

    pub fn mode(&self) -> &GalleryMode {
        &self.mode
    }

    pub fn set_mode(&mut self, mode: GalleryMode) {
        self.mode = mode;
        self.slideshow_timer = 0.0;
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.25, 4.0);
        self.thumbnail_size = 150.0 * self.zoom;
        self.update_columns();
    }

    pub fn zoom_in(&mut self) {
        self.set_zoom(self.zoom * 1.25);
    }

    pub fn zoom_out(&mut self) {
        self.set_zoom(self.zoom / 1.25);
    }

    pub fn thumbnail_size(&self) -> f32 {
        self.thumbnail_size
    }

    pub fn columns(&self) -> usize {
        self.columns
    }

    pub fn update_columns(&mut self) {
        if self.thumbnail_size > 0.0 {
            self.columns = (self.bounds.width / self.thumbnail_size) as usize;
            if self.columns == 0 {
                self.columns = 1;
            }
        }
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset;
    }

    pub fn scroll(&mut self, delta: i32) {
        let max_scroll = self.items.len() / self.columns;
        self.scroll_offset = (self.scroll_offset as i32 + delta)
            .max(0)
            .min(max_scroll as i32) as usize;
    }

    pub fn slideshow_interval(&self) -> f32 {
        self.slideshow_interval
    }

    pub fn set_slideshow_interval(&mut self, interval: f32) {
        self.slideshow_interval = interval.max(0.5);
    }

    pub fn update_slideshow(&mut self, dt: f32) {
        if self.mode == GalleryMode::Slideshow {
            self.slideshow_timer += dt;
            if self.slideshow_timer >= self.slideshow_interval {
                self.slideshow_timer = 0.0;
                self.next();
            }
        }
    }

    pub fn next(&mut self) {
        if let Some(idx) = self.selected_index {
            if idx + 1 < self.items.len() {
                self.selected_index = Some(idx + 1);
            } else {
                self.selected_index = Some(0);
            }
        }
    }

    pub fn previous(&mut self) {
        if let Some(idx) = self.selected_index {
            if idx > 0 {
                self.selected_index = Some(idx - 1);
            } else {
                self.selected_index = Some(self.items.len() - 1);
            }
        }
    }

    pub fn move_left(&mut self) {
        if let Some(idx) = self.selected_index {
            if idx > 0 {
                self.selected_index = Some(idx - 1);
            }
        }
    }

    pub fn move_right(&mut self) {
        if let Some(idx) = self.selected_index {
            if idx + 1 < self.items.len() {
                self.selected_index = Some(idx + 1);
            }
        }
    }

    pub fn move_up(&mut self) {
        if let Some(idx) = self.selected_index {
            let new_idx = idx.saturating_sub(self.columns);
            self.selected_index = Some(new_idx);
        }
    }

    pub fn move_down(&mut self) {
        if let Some(idx) = self.selected_index {
            let new_idx = (idx + self.columns).min(self.items.len() - 1);
            self.selected_index = Some(new_idx);
        }
    }
}

impl Default for GalleryView {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for GalleryView {
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

    fn update(&mut self, dt: f32) {
        self.update_slideshow(dt);
    }

    fn handle_key_down(&mut self, key: u32) -> bool {
        match key {
            37 => {
                self.move_left();
                true
            }
            39 => {
                self.move_right();
                true
            }
            38 => {
                self.move_up();
                true
            }
            40 => {
                self.move_down();
                true
            }
            33 => {
                self.scroll(-1);
                true
            }
            34 => {
                self.scroll(1);
                true
            }
            _ => false,
        }
    }

    fn handle_mouse_button_down(&mut self, x: f32, y: f32) -> bool {
        if !self.bounds.contains(x, y) {
            return false;
        }

        let item_height = self.thumbnail_size + 20.0;
        let col = ((x - self.bounds.x) / self.thumbnail_size) as usize;
        let row = ((y - self.bounds.y) / item_height) as usize + self.scroll_offset;
        let index = row * self.columns + col;

        if index < self.items.len() {
            self.selected_index = Some(index);
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gallery_view_new() {
        let view = GalleryView::new();
        assert!(view.items().is_empty());
        assert_eq!(*view.mode(), GalleryMode::Grid);
        assert_eq!(view.zoom(), 1.0);
    }

    #[test]
    fn test_gallery_view_set_items() {
        let mut view = GalleryView::new();
        let items = vec![
            GalleryItem::new(PathBuf::from("image1.jpg")),
            GalleryItem::new(PathBuf::from("image2.jpg")),
            GalleryItem::new(PathBuf::from("image3.jpg")),
        ];
        view.set_items(items);
        
        assert_eq!(view.items().len(), 3);
        assert_eq!(view.selected_index(), Some(0));
    }

    #[test]
    fn test_gallery_view_navigation() {
        let mut view = GalleryView::new();
        let items = vec![
            GalleryItem::new(PathBuf::from("image1.jpg")),
            GalleryItem::new(PathBuf::from("image2.jpg")),
            GalleryItem::new(PathBuf::from("image3.jpg")),
            GalleryItem::new(PathBuf::from("image4.jpg")),
        ];
        view.set_items(items);
        
        view.move_right();
        assert_eq!(view.selected_index(), Some(1));
        
        view.move_right();
        assert_eq!(view.selected_index(), Some(2));
        
        view.move_left();
        assert_eq!(view.selected_index(), Some(1));
    }

    #[test]
    fn test_gallery_view_next_previous() {
        let mut view = GalleryView::new();
        let items = vec![
            GalleryItem::new(PathBuf::from("image1.jpg")),
            GalleryItem::new(PathBuf::from("image2.jpg")),
            GalleryItem::new(PathBuf::from("image3.jpg")),
        ];
        view.set_items(items);
        
        view.next();
        assert_eq!(view.selected_index(), Some(1));
        
        view.next();
        assert_eq!(view.selected_index(), Some(2));
        
        view.next();
        assert_eq!(view.selected_index(), Some(0)); // Wraps around
        
        view.previous();
        assert_eq!(view.selected_index(), Some(2));
    }

    #[test]
    fn test_gallery_view_zoom() {
        let mut view = GalleryView::new();
        
        view.set_zoom(2.0);
        assert_eq!(view.zoom(), 2.0);
        assert!(view.thumbnail_size() > 150.0);
        
        view.zoom_in();
        let zoom_after_in = view.zoom();
        assert!(zoom_after_in > 2.0);
        
        view.zoom_out();
        let zoom_after_out = view.zoom();
        assert!(zoom_after_out < zoom_after_in);
        assert!((zoom_after_out - 2.0).abs() < 0.01); // Returns to approximately 2.0
    }

    #[test]
    fn test_gallery_view_zoom_limits() {
        let mut view = GalleryView::new();
        
        view.set_zoom(0.1);
        assert_eq!(view.zoom(), 0.25); // Min zoom
        
        view.set_zoom(10.0);
        assert_eq!(view.zoom(), 4.0); // Max zoom
    }

    #[test]
    fn test_gallery_view_scroll() {
        let mut view = GalleryView::new();
        let items: Vec<GalleryItem> = (0..20)
            .map(|i| GalleryItem::new(PathBuf::from(format!("image{}.jpg", i))))
            .collect();
        view.set_items(items);
        
        view.scroll(1);
        assert_eq!(view.scroll_offset(), 1);
        
        view.scroll(-1);
        assert_eq!(view.scroll_offset(), 0);
    }

    #[test]
    fn test_gallery_view_mode() {
        let mut view = GalleryView::new();
        
        assert_eq!(*view.mode(), GalleryMode::Grid);
        
        view.set_mode(GalleryMode::Slideshow);
        assert_eq!(*view.mode(), GalleryMode::Slideshow);
    }
}

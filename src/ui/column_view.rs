use std::path::PathBuf;

use super::components::{Component, ComponentState, Rect};

#[derive(Debug, Clone)]
pub struct ColumnItem {
    pub name: String,
    pub path: PathBuf,
    pub is_directory: bool,
    pub size: u64,
    pub modified: Option<chrono::DateTime<chrono::Utc>>,
}

impl ColumnItem {
    pub fn new(name: &str, path: PathBuf, is_directory: bool) -> Self {
        Self {
            name: name.to_string(),
            path,
            is_directory,
            size: 0,
            modified: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Column {
    pub items: Vec<ColumnItem>,
    pub selected_index: Option<usize>,
    pub scroll_offset: usize,
    pub width: f32,
}

impl Column {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected_index: None,
            scroll_offset: 0,
            width: 200.0,
        }
    }

    pub fn set_items(&mut self, items: Vec<ColumnItem>) {
        self.items = items;
        self.selected_index = if self.items.is_empty() {
            None
        } else {
            Some(0)
        };
        self.scroll_offset = 0;
    }

    pub fn select(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected_index = Some(index);
        }
    }

    pub fn selected_item(&self) -> Option<&ColumnItem> {
        self.selected_index.and_then(|i| self.items.get(i))
    }
}

impl Default for Column {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ColumnView {
    id: String,
    bounds: Rect,
    state: ComponentState,
    visible: bool,
    columns: Vec<Column>,
    active_column: usize,
    column_width: f32,
    preview_visible: bool,
}

impl ColumnView {
    pub fn new() -> Self {
        Self {
            id: "column_view".to_string(),
            bounds: Rect::default(),
            state: ComponentState::Normal,
            visible: true,
            columns: vec![Column::new()],
            active_column: 0,
            column_width: 200.0,
            preview_visible: true,
        }
    }

    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    pub fn columns_mut(&mut self) -> &mut Vec<Column> {
        &mut self.columns
    }

    pub fn active_column(&self) -> usize {
        self.active_column
    }

    pub fn set_active_column(&mut self, index: usize) {
        if index < self.columns.len() {
            self.active_column = index;
        }
    }

    pub fn set_root(&mut self, items: Vec<ColumnItem>) {
        self.columns.clear();
        let mut column = Column::new();
        column.set_items(items);
        self.columns.push(column);
        self.active_column = 0;
    }

    pub fn expand_folder(&mut self, column_index: usize, item_index: usize) {
        if column_index >= self.columns.len() {
            return;
        }

        let item = match self.columns[column_index].items.get(item_index) {
            Some(item) => item.clone(),
            None => return,
        };

        if !item.is_directory {
            return;
        }

        // Remove columns after the clicked one
        self.columns.truncate(column_index + 1);

        // Add new column with folder contents
        let mut new_column = Column::new();
        // In real implementation, we would read the directory here
        // For now, we just create an empty column
        self.columns.push(new_column);
        self.active_column = self.columns.len() - 1;
    }

    pub fn go_back(&mut self) {
        if self.active_column > 0 {
            self.columns.truncate(self.active_column);
            self.active_column -= 1;
        }
    }

    pub fn selected_path(&self) -> Option<PathBuf> {
        self.columns
            .last()
            .and_then(|col| col.selected_item())
            .map(|item| item.path.clone())
    }

    pub fn column_width(&self) -> f32 {
        self.column_width
    }

    pub fn set_column_width(&mut self, width: f32) {
        self.column_width = width.clamp(150.0, 400.0);
    }

    pub fn preview_visible(&self) -> bool {
        self.preview_visible
    }

    pub fn set_preview_visible(&mut self, visible: bool) {
        self.preview_visible = visible;
    }

    pub fn toggle_preview(&mut self) {
        self.preview_visible = !self.preview_visible;
    }

    pub fn move_left(&mut self) {
        if self.active_column > 0 {
            self.active_column -= 1;
        }
    }

    pub fn move_right(&mut self) {
        if self.active_column + 1 < self.columns.len() {
            self.active_column += 1;
        }
    }

    pub fn move_up(&mut self) {
        if let Some(col) = self.columns.get_mut(self.active_column) {
            if let Some(idx) = col.selected_index {
                if idx > 0 {
                    col.selected_index = Some(idx - 1);
                }
            }
        }
    }

    pub fn move_down(&mut self) {
        if let Some(col) = self.columns.get_mut(self.active_column) {
            let max = col.items.len();
            if let Some(idx) = col.selected_index {
                if idx + 1 < max {
                    col.selected_index = Some(idx + 1);
                }
            } else if max > 0 {
                col.selected_index = Some(0);
            }
        }
    }

    pub fn expand_selected(&mut self) {
        let col_index = self.active_column;
        if let Some(item_index) = self.columns[col_index].selected_index {
            self.expand_folder(col_index, item_index);
        }
    }
}

impl Default for ColumnView {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for ColumnView {
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
            13 | 39 => {
                self.expand_selected();
                true
            }
            8 | 27 => {
                self.go_back();
                true
            }
            _ => false,
        }
    }

    fn handle_mouse_button_down(&mut self, x: f32, y: f32) -> bool {
        if !self.bounds.contains(x, y) {
            return false;
        }

        let column_x = self.bounds.x;
        let item_height = 32.0;

        // Find the clicked column and item first
        let mut clicked_col_idx = None;
        let mut clicked_item_idx = None;
        let mut is_directory = false;

        for (col_idx, col) in self.columns.iter().enumerate() {
            let col_right = column_x + self.column_width * (col_idx as f32 + 1.0);
            
            if x >= column_x + self.column_width * col_idx as f32 && x < col_right {
                let item_index = ((y - self.bounds.y) / item_height) as usize;
                if item_index < col.items.len() {
                    clicked_col_idx = Some(col_idx);
                    clicked_item_idx = Some(item_index);
                    is_directory = col.items[item_index].is_directory;
                    break;
                }
            }
        }

        if let (Some(col_idx), Some(item_idx)) = (clicked_col_idx, clicked_item_idx) {
            self.active_column = col_idx;
            self.columns[col_idx].select(item_idx);
            
            if is_directory {
                self.expand_folder(col_idx, item_idx);
            }
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_view_new() {
        let view = ColumnView::new();
        assert_eq!(view.columns().len(), 1);
        assert_eq!(view.active_column(), 0);
    }

    #[test]
    fn test_column_view_set_root() {
        let mut view = ColumnView::new();
        let items = vec![
            ColumnItem::new("folder1", PathBuf::from("/folder1"), true),
            ColumnItem::new("file1.txt", PathBuf::from("/file1.txt"), false),
        ];
        view.set_root(items);
        
        assert_eq!(view.columns().len(), 1);
        assert_eq!(view.columns()[0].items.len(), 2);
    }

    #[test]
    fn test_column_view_expand_folder() {
        let mut view = ColumnView::new();
        let items = vec![
            ColumnItem::new("folder1", PathBuf::from("/folder1"), true),
            ColumnItem::new("file1.txt", PathBuf::from("/file1.txt"), false),
        ];
        view.set_root(items);
        
        view.expand_folder(0, 0);
        
        assert_eq!(view.columns().len(), 2);
        assert_eq!(view.active_column(), 1);
    }

    #[test]
    fn test_column_view_go_back() {
        let mut view = ColumnView::new();
        let items = vec![
            ColumnItem::new("folder1", PathBuf::from("/folder1"), true),
        ];
        view.set_root(items);
        view.expand_folder(0, 0);
        
        assert_eq!(view.active_column(), 1);
        
        view.go_back();
        assert_eq!(view.active_column(), 0);
        assert_eq!(view.columns().len(), 1);
    }

    #[test]
    fn test_column_view_navigation() {
        let mut view = ColumnView::new();
        let items = vec![
            ColumnItem::new("folder1", PathBuf::from("/folder1"), true),
            ColumnItem::new("folder2", PathBuf::from("/folder2"), true),
            ColumnItem::new("folder3", PathBuf::from("/folder3"), true),
        ];
        view.set_root(items);
        
        // After set_root, selected_index is Some(0)
        assert_eq!(view.columns()[0].selected_index, Some(0));
        
        view.move_down();
        assert_eq!(view.columns()[0].selected_index, Some(1));
        
        view.move_down();
        assert_eq!(view.columns()[0].selected_index, Some(2));
        
        view.move_up();
        assert_eq!(view.columns()[0].selected_index, Some(1));
        
        view.move_up();
        assert_eq!(view.columns()[0].selected_index, Some(0));
    }

    #[test]
    fn test_column_view_selected_path() {
        let mut view = ColumnView::new();
        let items = vec![
            ColumnItem::new("folder1", PathBuf::from("/folder1"), true),
            ColumnItem::new("file1.txt", PathBuf::from("/file1.txt"), false),
        ];
        view.set_root(items);
        
        let path = view.selected_path();
        assert!(path.is_some());
    }

    #[test]
    fn test_column_view_toggle_preview() {
        let mut view = ColumnView::new();
        assert!(view.preview_visible());
        
        view.toggle_preview();
        assert!(!view.preview_visible());
        
        view.toggle_preview();
        assert!(view.preview_visible());
    }
}

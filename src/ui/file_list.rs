use crate::ui::components::{Component, ComponentState, Rect};
use std::path::PathBuf;

const ROW_HEIGHT: f32 = 36.0;
const HEADER_HEIGHT: f32 = 28.0;
const MIN_COLUMN_WIDTH: f32 = 80.0;

#[derive(Debug, Clone, PartialEq)]
pub enum SortColumn {
    Name,
    Type,
    Size,
    Modified,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectionMode {
    None,
    Single,
    Multiple,
}

#[derive(Debug, Clone)]
pub struct FileItem {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<u64>,
    pub extension: Option<String>,
}

impl FileItem {
    pub fn new(name: &str, path: PathBuf, is_dir: bool) -> Self {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_string());

        Self {
            name: name.to_string(),
            path,
            is_dir,
            size: 0,
            modified: None,
            extension,
        }
    }

    pub fn file_type(&self) -> &str {
        if self.is_dir {
            "Folder"
        } else {
            match self.extension.as_deref() {
                Some(ext) => ext,
                None => "File",
            }
        }
    }

    pub fn display_size(&self) -> String {
        if self.is_dir {
            String::new()
        } else if self.size < 1024 {
            format!("{} B", self.size)
        } else if self.size < 1024 * 1024 {
            format!("{:.1} KB", self.size as f64 / 1024.0)
        } else if self.size < 1024 * 1024 * 1024 {
            format!("{:.1} MB", self.size as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", self.size as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }

    pub fn display_modified(&self) -> String {
        self.modified
            .map(|ts| {
                let dt = chrono::DateTime::from_timestamp(ts as i64, 0)
                    .unwrap_or_default();
                dt.format("%Y-%m-%d %H:%M").to_string()
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct Column {
    pub title: String,
    pub width: f32,
    pub sort_column: SortColumn,
    pub visible: bool,
}

impl Column {
    pub fn new(title: &str, width: f32, sort_column: SortColumn) -> Self {
        Self {
            title: title.to_string(),
            width,
            sort_column,
            visible: true,
        }
    }
}

#[derive(Debug)]
pub struct FileList {
    id: String,
    bounds: Rect,
    state: ComponentState,
    visible: bool,
    items: Vec<FileItem>,
    columns: Vec<Column>,
    sort_column: SortColumn,
    sort_order: SortOrder,
    selected_indices: Vec<usize>,
    selection_mode: SelectionMode,
    scroll_offset: f64,
    hovered_index: Option<usize>,
    active: bool,
}

impl FileList {
    pub fn new(id: &str, bounds: Rect) -> Self {
        let columns = vec![
            Column::new("Name", 300.0, SortColumn::Name),
            Column::new("Type", 100.0, SortColumn::Type),
            Column::new("Size", 100.0, SortColumn::Size),
            Column::new("Modified", 150.0, SortColumn::Modified),
        ];

        Self {
            id: id.to_string(),
            bounds,
            state: ComponentState::Normal,
            visible: true,
            items: Vec::new(),
            columns,
            sort_column: SortColumn::Name,
            sort_order: SortOrder::Ascending,
            selected_indices: Vec::new(),
            selection_mode: SelectionMode::Single,
            scroll_offset: 0.0,
            hovered_index: None,
            active: true,
        }
    }

    pub fn items(&self) -> &[FileItem] {
        &self.items
    }

    pub fn set_items(&mut self, items: Vec<FileItem>) {
        self.items = items;
        self.sort_items();
        self.selected_indices.clear();
    }

    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    pub fn columns_mut(&mut self) -> &mut Vec<Column> {
        &mut self.columns
    }

    pub fn sort_column(&self) -> &SortColumn {
        &self.sort_column
    }

    pub fn sort_order(&self) -> &SortOrder {
        &self.sort_order
    }

    pub fn selected_indices(&self) -> &[usize] {
        &self.selected_indices
    }

    pub fn selected_items(&self) -> Vec<&FileItem> {
        self.selected_indices
            .iter()
            .filter_map(|&i| self.items.get(i))
            .collect()
    }

    pub fn set_sort(&mut self, column: SortColumn, order: SortOrder) {
        self.sort_column = column;
        self.sort_order = order;
        self.sort_items();
    }

    pub fn toggle_sort(&mut self, column: SortColumn) {
        if self.sort_column == column {
            self.sort_order = match self.sort_order {
                SortOrder::Ascending => SortOrder::Descending,
                SortOrder::Descending => SortOrder::Ascending,
            };
        } else {
            self.sort_column = column;
            self.sort_order = SortOrder::Ascending;
        }
        self.sort_items();
    }

    fn sort_items(&mut self) {
        self.items.sort_by(|a, b| {
            let cmp = match self.sort_column {
                SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortColumn::Type => {
                    if a.is_dir == b.is_dir {
                        a.file_type().cmp(b.file_type())
                    } else if a.is_dir {
                        std::cmp::Ordering::Less
                    } else {
                        std::cmp::Ordering::Greater
                    }
                }
                SortColumn::Size => a.size.cmp(&b.size),
                SortColumn::Modified => a.modified.unwrap_or(0).cmp(&b.modified.unwrap_or(0)),
            };

            match self.sort_order {
                SortOrder::Ascending => cmp,
                SortOrder::Descending => cmp.reverse(),
            }
        });
    }

    pub fn select(&mut self, index: usize, multi: bool) {
        if !multi {
            self.selected_indices.clear();
        }

        if self.selected_indices.contains(&index) {
            self.selected_indices.retain(|&i| i != index);
        } else {
            self.selected_indices.push(index);
        }
    }

    pub fn select_all(&mut self) {
        self.selected_indices = (0..self.items.len()).collect();
    }

    pub fn clear_selection(&mut self) {
        self.selected_indices.clear();
    }

    pub fn item_at(&self, x: f32, y: f32) -> Option<usize> {
        if !self.bounds.contains(x, y) {
            return None;
        }

        let content_y = self.bounds.y + HEADER_HEIGHT;
        if y < content_y {
            return None;
        }

        let row = ((y - content_y + self.scroll_offset as f32) / ROW_HEIGHT) as usize;
        if row < self.items.len() {
            Some(row)
        } else {
            None
        }
    }

    pub fn column_at(&self, x: f32) -> Option<usize> {
        if !self.bounds.contains(x, self.bounds.y) {
            return None;
        }

        let mut col_x = self.bounds.x;
        for (i, col) in self.columns.iter().enumerate() {
            if !col.visible {
                continue;
            }
            if x >= col_x && x < col_x + col.width {
                return Some(i);
            }
            col_x += col.width;
        }

        None
    }

    pub fn visible_rows(&self) -> usize {
        let content_height = self.bounds.height - HEADER_HEIGHT;
        (content_height / ROW_HEIGHT) as usize + 1
    }

    pub fn total_height(&self) -> f32 {
        self.items.len() as f32 * ROW_HEIGHT
    }

    pub fn scroll(&mut self, delta: f64) {
        let max_scroll = (self.total_height() as f64 - (self.bounds.height - HEADER_HEIGHT) as f64).max(0.0);
        self.scroll_offset = (self.scroll_offset + delta).clamp(0.0, max_scroll);
    }

    pub fn scroll_to(&mut self, index: usize) {
        let row_top = index as f64 * ROW_HEIGHT as f64;
        let row_bottom = row_top + ROW_HEIGHT as f64;
        let view_top = self.scroll_offset;
        let view_bottom = self.scroll_offset + (self.bounds.height - HEADER_HEIGHT) as f64;

        if row_top < view_top {
            self.scroll_offset = row_top;
        } else if row_bottom > view_bottom {
            self.scroll_offset = row_bottom - (self.bounds.height - HEADER_HEIGHT) as f64;
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}

impl Component for FileList {
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
            self.hovered_index = self.item_at(x, y);
            self.set_state(ComponentState::Hovered);
            return true;
        }

        self.hovered_index = None;
        self.set_state(ComponentState::Normal);
        false
    }

    fn handle_mouse_button_down(&mut self, x: f32, y: f32) -> bool {
        if !self.bounds.contains(x, y) {
            return false;
        }

        if y < self.bounds.y + HEADER_HEIGHT {
            if let Some(col_index) = self.column_at(x) {
                let col = &self.columns[col_index];
                self.toggle_sort(col.sort_column.clone());
            }
            return true;
        }

        if let Some(index) = self.item_at(x, y) {
            let ctrl = false;
            self.select(index, ctrl);
            self.set_state(ComponentState::Pressed);
            return true;
        }

        false
    }

    fn handle_mouse_button_up(&mut self, _x: f32, _y: f32) -> bool {
        if *self.state() == ComponentState::Pressed {
            self.set_state(ComponentState::Hovered);
            return true;
        }

        false
    }

    fn handle_key_down(&mut self, key: u32) -> bool {
        match key {
            38 => {
                let first = self.selected_indices.first().copied();
                if let Some(first) = first {
                    if first > 0 {
                        self.clear_selection();
                        self.select(first - 1, false);
                        self.scroll_to(first - 1);
                    }
                } else if !self.items.is_empty() {
                    self.select(0, false);
                    self.scroll_to(0);
                }
                true
            }
            40 => {
                let last = self.selected_indices.last().copied();
                if let Some(last) = last {
                    if last + 1 < self.items.len() {
                        self.clear_selection();
                        self.select(last + 1, false);
                        self.scroll_to(last + 1);
                    }
                } else if !self.items.is_empty() {
                    self.select(0, false);
                    self.scroll_to(0);
                }
                true
            }
            36 => {
                if !self.items.is_empty() {
                    self.clear_selection();
                    self.select(0, false);
                    self.scroll_to(0);
                }
                true
            }
            35 => {
                if !self.items.is_empty() {
                    self.clear_selection();
                    self.select(self.items.len() - 1, false);
                    self.scroll_to(self.items.len() - 1);
                }
                true
            }
            65 => {
                if self.bounds.contains(self.bounds.x, self.bounds.y) {
                    self.select_all();
                }
                self.bounds.contains(self.bounds.x, self.bounds.y)
            }
            _ => false,
        }
    }

    fn handle_char_input(&mut self, _ch: char) -> bool {
        false
    }
}

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LayoutMode {
    Single,
    DualVertical,
    DualHorizontal,
    TripleLeft,
    TripleRight,
    Quad,
    Cascade,
}

impl LayoutMode {
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Self::Single,
            1 => Self::DualVertical,
            2 => Self::DualHorizontal,
            3 => Self::TripleLeft,
            4 => Self::TripleRight,
            5 => Self::Quad,
            6 => Self::Cascade,
            _ => Self::Single,
        }
    }

    pub fn to_index(&self) -> usize {
        match self {
            Self::Single => 0,
            Self::DualVertical => 1,
            Self::DualHorizontal => 2,
            Self::TripleLeft => 3,
            Self::TripleRight => 4,
            Self::Quad => 5,
            Self::Cascade => 6,
        }
    }

    pub fn panel_count(&self) -> usize {
        match self {
            Self::Single => 1,
            Self::DualVertical | Self::DualHorizontal => 2,
            Self::TripleLeft | Self::TripleRight => 3,
            Self::Quad => 4,
            Self::Cascade => 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PanelState {
    pub id: usize,
    pub path: PathBuf,
    pub selected_files: Vec<String>,
    pub scroll_offset: f64,
    pub sort_by: SortBy,
    pub sort_ascending: bool,
    pub column_widths: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortBy {
    Name,
    Size,
    Type,
    Modified,
}

impl Default for PanelState {
    fn default() -> Self {
        Self {
            id: 0,
            path: PathBuf::from("C:\\"),
            selected_files: Vec::new(),
            scroll_offset: 0.0,
            sort_by: SortBy::Name,
            sort_ascending: true,
            column_widths: vec![200.0, 100.0, 100.0, 150.0],
        }
    }
}

#[derive(Debug, Clone)]
pub struct TabState {
    pub id: usize,
    pub panels: Vec<PanelState>,
    pub active_panel: usize,
    pub layout: LayoutMode,
    pub is_pinned: bool,
}

impl Default for TabState {
    fn default() -> Self {
        Self {
            id: 0,
            panels: vec![PanelState::default()],
            active_panel: 0,
            layout: LayoutMode::Single,
            is_pinned: false,
        }
    }
}

impl TabState {
    pub fn update(&mut self, _dt: f32) {
        // Update tab state
    }
}

#[derive(Debug)]
pub struct AppState {
    pub tabs: Vec<TabState>,
    pub active_tab: usize,
    pub theme: Theme,
    pub sidebar_visible: bool,
    pub sidebar_position: SidebarPosition,
    pub sidebar_width: f32,
    pub status_bar_visible: bool,
    pub vim_mode: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SidebarPosition {
    Left,
    Right,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            tabs: vec![TabState::default()],
            active_tab: 0,
            theme: Theme::Dark,
            sidebar_visible: true,
            sidebar_position: SidebarPosition::Left,
            sidebar_width: 250.0,
            status_bar_visible: true,
            vim_mode: false,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn active_tab(&self) -> &TabState {
        &self.tabs[self.active_tab]
    }

    pub fn active_tab_mut(&mut self) -> &mut TabState {
        &mut self.tabs[self.active_tab]
    }

    pub fn active_panel(&self) -> &PanelState {
        let tab = self.active_tab();
        &tab.panels[tab.active_panel]
    }

    pub fn active_panel_mut(&mut self) -> &mut PanelState {
        let tab = &mut self.tabs[self.active_tab];
        &mut tab.panels[tab.active_panel]
    }
}

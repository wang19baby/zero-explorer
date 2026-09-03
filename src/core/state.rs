use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    System,
}

/// 布局模板 - 参考 Tessoa 的 12 种布局
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutMode {
    // 一格
    Single,
    // 两格
    DualVertical,    // 双栏·左右
    DualHorizontal,  // 双栏·上下
    // 三格
    TripleLeft,           // 三栏·左一右二（保留兼容）
    TripleRight,          // 三栏·左二右一（保留兼容）
    TripleHorizontal,     // 三栏·横排
    TripleTopTwoBottom,   // 三栏·上二下一
    TripleTopOneBottom,   // 三栏·上一下二
    // 四格
    Quad,                 // 四栏·田字格
    QuadHorizontal,       // 四栏·横排
    QuadLeftOneRightThree, // 四栏·左一右三
    QuadTopOneBottomThree, // 四栏·上一下三
    // 其他
    Cascade,
}

/// 布局模板定义
#[derive(Debug, Clone)]
pub struct LayoutTemplate {
    pub name: &'static str,
    pub mode: LayoutMode,
    pub panel_count: usize,
    pub description: &'static str,
}

impl LayoutTemplate {
    /// 获取所有 12 种布局模板
    pub fn all_templates() -> Vec<LayoutTemplate> {
        vec![
            // 一格
            LayoutTemplate {
                name: "单栏",
                mode: LayoutMode::Single,
                panel_count: 1,
                description: "单个窗格",
            },
            // 两格
            LayoutTemplate {
                name: "双栏·左右",
                mode: LayoutMode::DualVertical,
                panel_count: 2,
                description: "左右分栏",
            },
            LayoutTemplate {
                name: "双栏·上下",
                mode: LayoutMode::DualHorizontal,
                panel_count: 2,
                description: "上下分栏",
            },
            // 三格
            LayoutTemplate {
                name: "三栏·横排",
                mode: LayoutMode::TripleHorizontal,
                panel_count: 3,
                description: "三个窗格水平排列",
            },
            LayoutTemplate {
                name: "三栏·左一右二",
                mode: LayoutMode::TripleLeft,
                panel_count: 3,
                description: "左侧一个，右侧两个上下排列",
            },
            LayoutTemplate {
                name: "三栏·左二右一",
                mode: LayoutMode::TripleRight,
                panel_count: 3,
                description: "左侧两个上下排列，右侧一个",
            },
            LayoutTemplate {
                name: "三栏·上一下二",
                mode: LayoutMode::TripleTopOneBottom,
                panel_count: 3,
                description: "上方一个，下方两个左右排列",
            },
            LayoutTemplate {
                name: "三栏·上二下一",
                mode: LayoutMode::TripleTopTwoBottom,
                panel_count: 3,
                description: "上方两个左右排列，下方一个",
            },
            // 四格
            LayoutTemplate {
                name: "四栏·田字格",
                mode: LayoutMode::Quad,
                panel_count: 4,
                description: "四个窗格等分",
            },
            LayoutTemplate {
                name: "四栏·横排",
                mode: LayoutMode::QuadHorizontal,
                panel_count: 4,
                description: "四个窗格水平排列",
            },
            LayoutTemplate {
                name: "四栏·左一右三",
                mode: LayoutMode::QuadLeftOneRightThree,
                panel_count: 4,
                description: "左侧一个，右侧三个",
            },
            LayoutTemplate {
                name: "四栏·上一下三",
                mode: LayoutMode::QuadTopOneBottomThree,
                panel_count: 4,
                description: "上方一个，下方三个",
            },
        ]
    }

    /// 根据名称查找模板
    pub fn find_by_name(name: &str) -> Option<LayoutTemplate> {
        Self::all_templates().into_iter().find(|t| t.name == name)
    }

    /// 根据 mode 查找模板
    pub fn find_by_mode(mode: &LayoutMode) -> Option<LayoutTemplate> {
        Self::all_templates().into_iter().find(|t| t.mode == *mode)
    }
}

impl LayoutMode {
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Self::Single,
            1 => Self::DualVertical,
            2 => Self::DualHorizontal,
            3 => Self::TripleLeft,
            4 => Self::TripleRight,
            5 => Self::TripleHorizontal,
            6 => Self::TripleTopTwoBottom,
            7 => Self::TripleTopOneBottom,
            8 => Self::Quad,
            9 => Self::QuadHorizontal,
            10 => Self::QuadLeftOneRightThree,
            11 => Self::QuadTopOneBottomThree,
            12 => Self::Cascade,
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
            Self::TripleHorizontal => 5,
            Self::TripleTopTwoBottom => 6,
            Self::TripleTopOneBottom => 7,
            Self::Quad => 8,
            Self::QuadHorizontal => 9,
            Self::QuadLeftOneRightThree => 10,
            Self::QuadTopOneBottomThree => 11,
            Self::Cascade => 12,
        }
    }

    pub fn panel_count(&self) -> usize {
        match self {
            Self::Single => 1,
            Self::DualVertical | Self::DualHorizontal => 2,
            Self::TripleLeft | Self::TripleRight | Self::TripleHorizontal
            | Self::TripleTopTwoBottom | Self::TripleTopOneBottom => 3,
            Self::Quad | Self::QuadHorizontal | Self::QuadLeftOneRightThree
            | Self::QuadTopOneBottomThree => 4,
            Self::Cascade => 2,
        }
    }

    /// 获取布局的显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Single => "单栏",
            Self::DualVertical => "双栏·左右",
            Self::DualHorizontal => "双栏·上下",
            Self::TripleLeft => "三栏·左一右二",
            Self::TripleRight => "三栏·左二右一",
            Self::TripleHorizontal => "三栏·横排",
            Self::TripleTopTwoBottom => "三栏·上二下一",
            Self::TripleTopOneBottom => "三栏·上一下二",
            Self::Quad => "四栏·田字格",
            Self::QuadHorizontal => "四栏·横排",
            Self::QuadLeftOneRightThree => "四栏·左一右三",
            Self::QuadTopOneBottomThree => "四栏·上一下三",
            Self::Cascade => "层叠",
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

/// 布局状态 - 参考 Tessoa 的布局管理系统
#[derive(Debug, Clone)]
pub struct LayoutState {
    /// 布局名称
    pub name: String,
    /// 布局模式
    pub mode: LayoutMode,
    /// 各窗格状态
    pub panels: Vec<PanelState>,
    /// 是否锁定（锁定后改动不写回）
    pub is_locked: bool,
    /// 是否已保存（有名字的布局）
    pub is_saved: bool,
    /// 创建时间
    pub created_at: Option<std::time::Instant>,
    /// 最后修改时间
    pub modified_at: Option<std::time::Instant>,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            name: String::new(),
            mode: LayoutMode::Single,
            panels: vec![PanelState::default()],
            is_locked: false,
            is_saved: false,
            created_at: None,
            modified_at: None,
        }
    }
}

impl LayoutState {
    /// 创建新的未命名布局
    pub fn new_unnamed() -> Self {
        Self {
            name: String::new(),
            mode: LayoutMode::Single,
            panels: vec![PanelState::default()],
            is_locked: false,
            is_saved: false,
            created_at: Some(std::time::Instant::now()),
            modified_at: Some(std::time::Instant::now()),
        }
    }

    /// 从模板创建布局
    pub fn from_template(template: &LayoutTemplate, panels: Vec<PanelState>) -> Self {
        Self {
            name: template.name.to_string(),
            mode: template.mode.clone(),
            panels,
            is_locked: false,
            is_saved: true,
            created_at: Some(std::time::Instant::now()),
            modified_at: Some(std::time::Instant::now()),
        }
    }

    /// 保存布局（给未命名布局起名字）
    pub fn save(&mut self, name: &str) {
        self.name = name.to_string();
        self.is_saved = true;
        self.modified_at = Some(std::time::Instant::now());
    }

    /// 锁定布局
    pub fn lock(&mut self) {
        self.is_locked = true;
    }

    /// 解锁布局
    pub fn unlock(&mut self) {
        self.is_locked = false;
    }

    /// 重新加载布局（丢弃改动，回到已保存的样子）
    pub fn reload(&self) -> LayoutState {
        // 如果是锁定的布局，返回原始状态
        // 如果是未保存的布局，返回默认状态
        if self.is_saved {
            self.clone()
        } else {
            LayoutState::new_unnamed()
        }
    }

    /// 检查是否有改动
    pub fn has_changes(&self, current: &LayoutState) -> bool {
        self.mode != current.mode || self.panels.len() != current.panels.len()
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
    /// 已保存的布局列表
    pub layouts: Vec<LayoutState>,
    /// 当前活跃的布局索引
    pub active_layout: Option<usize>,
    /// 未命名布局的保留槽
    pub unnamed_layout: Option<LayoutState>,
    /// 上次未命名的布局
    pub last_unnamed: Option<LayoutState>,
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
            layouts: Vec::new(),
            active_layout: None,
            unnamed_layout: Some(LayoutState::new_unnamed()),
            last_unnamed: None,
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

    /// 保存当前窗口为布局
    pub fn save_current_as_layout(&mut self, name: &str) -> usize {
        let layout = LayoutState {
            name: name.to_string(),
            mode: self.active_tab().layout.clone(),
            panels: self.active_tab().panels.clone(),
            is_locked: false,
            is_saved: true,
            created_at: Some(std::time::Instant::now()),
            modified_at: Some(std::time::Instant::now()),
        };

        self.layouts.push(layout);
        let index = self.layouts.len() - 1;
        self.active_layout = Some(index);
        index
    }

    /// 切换到指定布局
    pub fn switch_layout(&mut self, index: usize) -> bool {
        if index >= self.layouts.len() {
            return false;
        }

        // 保存当前布局（如果未锁定）
        if let Some(current_index) = self.active_layout {
            if current_index < self.layouts.len() {
                let current = &self.layouts[current_index];
                if !current.is_locked && current.is_saved {
                    // 自动保存当前布局
                    self.layouts[current_index].panels = self.active_tab().panels.clone();
                    self.layouts[current_index].mode = self.active_tab().layout.clone();
                    self.layouts[current_index].modified_at = Some(std::time::Instant::now());
                } else if !current.is_saved {
                    // 未命名布局，保存到 unnamed_layout
                    self.last_unnamed = Some(LayoutState {
                        name: String::new(),
                        mode: self.active_tab().layout.clone(),
                        panels: self.active_tab().panels.clone(),
                        is_locked: false,
                        is_saved: false,
                        created_at: current.created_at,
                        modified_at: Some(std::time::Instant::now()),
                    });
                }
            }
        }

        // 加载目标布局
        let target_mode = self.layouts[index].mode.clone();
        let target_panels = self.layouts[index].panels.clone();
        self.active_tab_mut().layout = target_mode;
        self.active_tab_mut().panels = target_panels;
        self.active_layout = Some(index);

        true
    }

    /// 锁定/解锁布局
    pub fn toggle_layout_lock(&mut self, index: usize) -> bool {
        if index >= self.layouts.len() {
            return false;
        }

        if self.layouts[index].is_locked {
            self.layouts[index].unlock();
        } else {
            self.layouts[index].lock();
        }

        true
    }

    /// 重新加载布局
    pub fn reload_layout(&mut self, index: usize) -> bool {
        if index >= self.layouts.len() {
            return false;
        }

        let layout = &self.layouts[index];
        if layout.is_saved {
            // 重新加载已保存的布局
            let mode = layout.mode.clone();
            let panels = layout.panels.clone();
            self.active_tab_mut().layout = mode;
            self.active_tab_mut().panels = panels;
            true
        } else {
            false
        }
    }

    /// 重命名布局
    pub fn rename_layout(&mut self, index: usize, new_name: &str) -> bool {
        if index >= self.layouts.len() {
            return false;
        }

        self.layouts[index].name = new_name.to_string();
        self.layouts[index].modified_at = Some(std::time::Instant::now());
        true
    }

    /// 复制布局
    pub fn duplicate_layout(&mut self, index: usize) -> Option<usize> {
        if index >= self.layouts.len() {
            return None;
        }

        let layout = self.layouts[index].clone();
        self.layouts.push(layout);
        Some(self.layouts.len() - 1)
    }

    /// 删除布局
    pub fn delete_layout(&mut self, index: usize) -> bool {
        if index >= self.layouts.len() {
            return false;
        }

        // 不能删除锁定的布局
        if self.layouts[index].is_locked {
            return false;
        }

        self.layouts.remove(index);

        // 调整活跃布局索引
        if let Some(active) = self.active_layout {
            if active == index {
                self.active_layout = None;
            } else if active > index {
                self.active_layout = Some(active - 1);
            }
        }

        true
    }

    /// 从模板创建布局
    pub fn create_layout_from_template(&mut self, template: &LayoutTemplate) -> usize {
        let panels: Vec<PanelState> = (0..template.panel_count)
            .enumerate()
            .map(|(i, _)| PanelState {
                id: i,
                ..Default::default()
            })
            .collect();

        let layout = LayoutState::from_template(template, panels);
        self.layouts.push(layout);
        let index = self.layouts.len() - 1;
        self.active_layout = Some(index);
        index
    }
}

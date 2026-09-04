use std::sync::Arc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use crate::core::event::EventDispatcher;
use crate::core::state::AppState;
use crate::fs::file_system::{LocalFileSystem, SortBy as FsSortBy};
use crate::ui::dual_panel::{DualPanelManager, PanelSnapshot, FileEntry, SortBy, ViewMode};
use crate::ui::folder_icons::FolderIconComposer;
use crate::ui::icons::FileIcon;
use crate::ui::renderer::GpuContext;
use crate::ui::shell_icons::IconSize;
use crate::ui::theme::Theme;
use crate::ui::virtual_scroll::VirtualScrollManager;

pub struct App {
    window: Option<Arc<Window>>,
    gpu: Option<GpuContext>,
    state: AppState,
    dispatcher: EventDispatcher,
    theme: Theme,
    mouse_x: f32,
    mouse_y: f32,
    hovered_area: HoveredArea,
    // UI state
    panel_mode: PanelMode,
    active_space: SpaceId,
    layout_type: LayoutType,
    sidebar_visible: bool,
    sidebar_width: f32,
    sidebar_position: SidebarPosition,
    preview_visible: bool,
    search_visible: bool,
    vim_help_visible: bool,
    selected_file_idx: Option<usize>,
    // Dragging state
    is_dragging_sidebar: bool,
    drag_start_x: f32,
    drag_start_width: f32,
    // Cascade state
    cascade_selected: [usize; 4],
    // Panel scroll state (up to 4 panels)
    panel_scroll_y: [f32; 4],
    panel_scroll_x: [f32; 4],
    // Keyboard modifiers
    modifiers: winit::keyboard::ModifiersState,
    // 双面板管理器
    dual_panel: DualPanelManager,
    // 虚拟化滚动管理器 (每个面板一个)
    virtual_scroll: [VirtualScrollManager; 4],
    // 文件夹图标合成器
    folder_icon_composer: FolderIconComposer,
    // 双击跟踪
    last_click_time: std::time::Instant,
    last_click_panel: usize,
    last_click_idx: usize,
    // 路径输入状态
    path_input_active: [bool; 2],     // 每个面板的路径输入是否激活
    path_input_text: [String; 2],     // 输入框中的文本
    path_input_cursor: [usize; 2],    // 光标位置
    // 标签页状态
    panel_tabs: [Vec<TabInfo>; 2],    // 每个面板的标签页列表
    active_tab_idx: [usize; 2],       // 每个面板的活跃标签索引
    tab_close_confirm: Option<(usize, usize)>, // (panel_idx, tab_idx) 确认关闭对话框
    tab_positions: [Vec<(f32, f32)>; 2], // 每个面板的标签位置 (x, width)
}

#[derive(Debug, Clone)]
struct TabInfo {
    name: String,
    path: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PanelMode {
    Panels,
    Cascade,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum LayoutType {
    Single,         // 1x1 单面板
    LeftRight,      // 1x2 左右分栏
    TopBottom,      // 2x1 上下分栏
    LeftMidRight,   // 1x3 左中右分栏
    Top2Bottom1,    // 上2下1
    Top1Bottom2,    // 上1下2
    FourGrid,       // 2x2 四面板
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SidebarPosition {
    Left,
    Right,
}

type SpaceId = &'static str;

#[derive(Debug, Clone, Copy, PartialEq)]
enum HoveredArea {
    None,
    Sidebar,
    TabBar,
    AddressBar,
    FileList,
    StatusBar,
}

impl App {
    fn text_y_centered(gpu: &GpuContext, container_y: f32, container_h: f32) -> f32 {
        let lh = gpu.line_height();
        let asc = gpu.ascent();
        container_y + (container_h - lh) / 2.0 + asc
    }

    /// Get which panel index the mouse position is over
    fn get_panel_at_position(&self, x: f32, y: f32) -> Option<usize> {
        let sw = self.window.as_ref()?.inner_size().width as f32;
        let sh = self.window.as_ref()?.inner_size().height as f32;
        let status_h = 30.0f32;
        let sidebar_w = if self.sidebar_visible { self.sidebar_width } else { 0.0 };
        
        let (main_x, main_w) = if self.sidebar_visible {
            match self.sidebar_position {
                SidebarPosition::Left => (sidebar_w + 1.0, sw - sidebar_w - 1.0),
                SidebarPosition::Right => (0.0, sw - sidebar_w - 1.0),
            }
        } else {
            (0.0, sw)
        };

        let divider_w = 4.0f32;
        let panel_count = match self.layout_type {
            LayoutType::Single => 1,
            LayoutType::LeftRight => 2,
            LayoutType::TopBottom => 2,
            LayoutType::LeftMidRight => 3,
            LayoutType::Top2Bottom1 => 3,
            LayoutType::Top1Bottom2 => 3,
            LayoutType::FourGrid => 4,
        };

        for idx in 0..panel_count {
            let (px, py, pw, ph) = match self.layout_type {
                LayoutType::Single => (main_x, 0.0, main_w, sh - status_h),
                LayoutType::LeftRight => {
                    let pw = (main_w - divider_w) / 2.0;
                    if idx == 0 { (main_x, 0.0, pw, sh - status_h) }
                    else { (main_x + pw + divider_w, 0.0, pw, sh - status_h) }
                }
                LayoutType::TopBottom => {
                    let ph = (sh - status_h - divider_w) / 2.0;
                    if idx == 0 { (main_x, 0.0, main_w, ph) }
                    else { (main_x, ph + divider_w, main_w, ph) }
                }
                LayoutType::LeftMidRight => {
                    let pw = (main_w - divider_w * 2.0) / 3.0;
                    (main_x + idx as f32 * (pw + divider_w), 0.0, pw, sh - status_h)
                }
                LayoutType::Top2Bottom1 => {
                    let pw = (main_w - divider_w) / 2.0;
                    let ph = (sh - status_h - divider_w) / 2.0;
                    match idx {
                        0 => (main_x, 0.0, pw, ph),
                        1 => (main_x + pw + divider_w, 0.0, pw, ph),
                        _ => (main_x, ph + divider_w, main_w, ph),
                    }
                }
                LayoutType::Top1Bottom2 => {
                    let pw = (main_w - divider_w) / 2.0;
                    let ph = (sh - status_h - divider_w) / 2.0;
                    match idx {
                        0 => (main_x, 0.0, main_w, ph),
                        _ => (main_x + (idx as f32 - 1.0) * (pw + divider_w), ph + divider_w, pw, ph),
                    }
                }
                LayoutType::FourGrid => {
                    let pw = (main_w - divider_w) / 2.0;
                    let ph = (sh - status_h - divider_w) / 2.0;
                    let row = idx / 2;
                    let col = idx % 2;
                    (main_x + col as f32 * (pw + divider_w), row as f32 * (ph + divider_w), pw, ph)
                }
            };
            if x >= px && x < px + pw && y >= py && y < py + ph {
                return Some(idx);
            }
        }
        None
    }

    pub fn new() -> Self {
        let mut app = Self {
            window: None,
            gpu: None,
            state: AppState::new(),
            dispatcher: EventDispatcher::new(),
            theme: Theme::light(),
            mouse_x: 0.0,
            mouse_y: 0.0,
            hovered_area: HoveredArea::None,
            panel_mode: PanelMode::Panels,
            active_space: "default",
            layout_type: LayoutType::LeftRight,
            sidebar_visible: true,
            sidebar_width: 200.0,
            sidebar_position: SidebarPosition::Left,
            preview_visible: false,
            search_visible: false,
            vim_help_visible: false,
            selected_file_idx: Some(2),
            is_dragging_sidebar: false,
            drag_start_x: 0.0,
            drag_start_width: 0.0,
            cascade_selected: [0; 4],
            panel_scroll_y: [0.0; 4],
            panel_scroll_x: [0.0; 4],
            modifiers: winit::keyboard::ModifiersState::empty(),
            dual_panel: DualPanelManager::new(),
            virtual_scroll: [
                VirtualScrollManager::new(28.0, 600.0),
                VirtualScrollManager::new(28.0, 600.0),
                VirtualScrollManager::new(28.0, 600.0),
                VirtualScrollManager::new(28.0, 600.0),
            ],
            folder_icon_composer: FolderIconComposer::new(std::num::NonZeroUsize::new(100).unwrap()),
            last_click_time: std::time::Instant::now(),
            last_click_panel: 0,
            last_click_idx: 0,
            path_input_active: [false; 2],
            path_input_text: [String::new(), String::new()],
            path_input_cursor: [0; 2],
            panel_tabs: [vec![], vec![]],
            active_tab_idx: [0; 2],
            tab_close_confirm: None,
            tab_positions: [vec![], vec![]],
        };

        // 初始化左面板文件数据 - 从磁盘读取
        let left_path = std::path::Path::new("D:\\work_space\\personal_workspace\\zero-explorer\\src");
        let left_files: Vec<FileEntry> = if let Ok(entries) = LocalFileSystem::read_dir_sorted(left_path, &FsSortBy::Name, true) {
            entries.into_iter().map(|fi| FileEntry {
                name: fi.name,
                path: fi.path.to_string_lossy().to_string(),
                is_dir: fi.file_type.is_dir(),
                size: fi.size,
                modified: fi.modified.map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()).unwrap_or(0),
                icon_id: 0,
            }).collect()
        } else {
            vec![]
        };
        app.dual_panel.set_left(PanelSnapshot {
            path: left_path.to_string_lossy().to_string(),
            files: left_files,
            selected_indices: vec![],
            focus_index: 0,
            scroll_offset: 0.0,
            sort_by: SortBy::Name,
            sort_descending: false,
            view_mode: ViewMode::Details,
        });

        // 初始化右面板文件数据 - 从磁盘读取
        let right_path = std::path::Path::new("D:\\work_space\\personal_workspace\\zero-explorer\\target");
        let right_files: Vec<FileEntry> = if let Ok(entries) = LocalFileSystem::read_dir_sorted(right_path, &FsSortBy::Name, true) {
            entries.into_iter().map(|fi| FileEntry {
                name: fi.name,
                path: fi.path.to_string_lossy().to_string(),
                is_dir: fi.file_type.is_dir(),
                size: fi.size,
                modified: fi.modified.map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()).unwrap_or(0),
                icon_id: 0,
            }).collect()
        } else {
            vec![]
        };
        app.dual_panel.set_right(PanelSnapshot {
            path: right_path.to_string_lossy().to_string(),
            files: right_files,
            selected_indices: vec![],
            focus_index: 0,
            scroll_offset: 0.0,
            sort_by: SortBy::Name,
            sort_descending: false,
            view_mode: ViewMode::Details,
        });

        // 初始化标签页
        app.panel_tabs[0] = vec![TabInfo {
            name: "src".to_string(),
            path: left_path.to_string_lossy().to_string(),
        }];
        app.panel_tabs[1] = vec![TabInfo {
            name: "target".to_string(),
            path: right_path.to_string_lossy().to_string(),
        }];

        app
    }

    /// 从目录加载文件列表
    fn load_directory(&self, path: &str) -> Vec<FileEntry> {
        let dir_path = std::path::Path::new(path);
        if let Ok(entries) = LocalFileSystem::read_dir_sorted(dir_path, &FsSortBy::Name, true) {
            entries.into_iter().map(|fi| FileEntry {
                name: fi.name,
                path: fi.path.to_string_lossy().to_string(),
                is_dir: fi.file_type.is_dir(),
                size: fi.size,
                modified: fi.modified
                    .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
                    .unwrap_or(0),
                icon_id: 0,
            }).collect()
        } else {
            vec![]
        }
    }

    /// 导航到指定路径（面板）
    fn navigate_to(&mut self, panel_idx: usize, path: &str) {
        let files = self.load_directory(path);
        let snapshot = PanelSnapshot {
            path: path.to_string(),
            files,
            selected_indices: vec![],
            focus_index: 0,
            scroll_offset: 0.0,
            sort_by: SortBy::Name,
            sort_descending: false,
            view_mode: ViewMode::Details,
        };
        if panel_idx == 0 {
            self.dual_panel.set_left(snapshot);
        } else {
            self.dual_panel.set_right(snapshot);
        }
        // 重置虚拟滚动
        if panel_idx < 4 {
            self.virtual_scroll[panel_idx] = VirtualScrollManager::new(28.0, 600.0);
        }
    }

    /// 返回上一级目录
    fn navigate_up(&mut self, panel_idx: usize) {
        let current_path = if panel_idx == 0 {
            self.dual_panel.left().path.clone()
        } else {
            self.dual_panel.right().path.clone()
        };
        if let Some(parent) = std::path::Path::new(&current_path).parent() {
            let parent_str = parent.to_string_lossy().to_string();
            self.navigate_to(panel_idx, &parent_str);
        }
    }

    /// 获取当前面板的目录名（用于标签页标题）
    fn current_dir_name(&self, panel_idx: usize) -> String {
        let path = if panel_idx == 0 {
            self.dual_panel.left().path.clone()
        } else {
            self.dual_panel.right().path.clone()
        };
        std::path::Path::new(&path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.clone())
    }

    /// 获取当前面板的文件数量
    fn file_count(&self, panel_idx: usize) -> usize {
        if panel_idx == 0 {
            self.dual_panel.left().files.len()
        } else {
            self.dual_panel.right().files.len()
        }
    }

    /// 获取当前面板的选中文件数量
    fn selected_count(&self, panel_idx: usize) -> usize {
        if panel_idx == 0 {
            self.dual_panel.left().selected_indices.len()
        } else {
            self.dual_panel.right().selected_indices.len()
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let event_loop = EventLoop::new()?;

        let window = Arc::new(
            WindowBuilder::new()
                .with_title("Zero Explorer")
                .with_inner_size(winit::dpi::LogicalSize::new(1200, 800))
                .with_min_inner_size(winit::dpi::LogicalSize::new(800, 600))
                .build(&event_loop)?,
        );

        let gpu = match GpuContext::new(window.clone()) {
            Ok(g) => {
                log::info!("GpuContext created successfully");
                g
            }
            Err(e) => {
                log::error!("Failed to create GpuContext: {:?}", e);
                return Err(e);
            }
        };

        self.window = Some(window.clone());
        self.gpu = Some(gpu);

        // 窗口创建后立即请求首次重绘
        log::info!("Window created, requesting first redraw");
        window.request_redraw();

        event_loop.run(move |event, target| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        target.exit();
                    }
                    WindowEvent::ModifiersChanged(modifiers) => {
                        self.modifiers = modifiers.state();
                    }
                    WindowEvent::Resized(size) => {
                        if let Some(gpu) = &mut self.gpu {
                            gpu.resize(size.width, size.height);
                        }
                        self.window.as_ref().unwrap().request_redraw();
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        self.mouse_x = position.x as f32;
                        self.mouse_y = position.y as f32;
                        
                        let old_hover = self.hovered_area;
                        let was_dragging = self.is_dragging_sidebar;
                        
                        // Handle sidebar dragging
                        if self.is_dragging_sidebar {
                            let delta = self.mouse_x - self.drag_start_x;
                            let new_width = self.drag_start_width + delta;
                            self.sidebar_width = new_width.clamp(150.0, 400.0);
                        }
                        
                        self.update_hovered_area();
                        
                        // 仅在 hover 变化或拖拽中时重绘
                        if self.hovered_area != old_hover || was_dragging {
                            self.window.as_ref().unwrap().request_redraw();
                        }
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        if state == winit::event::ElementState::Pressed {
                            self.handle_click(button);
                            
                            // Check if clicking on sidebar resize handle
                            let sw = self.window.as_ref().unwrap().inner_size().width as f32;
                            let sidebar_w = self.sidebar_width;
                            let handle_x_range = match self.sidebar_position {
                                SidebarPosition::Left => sidebar_w - 2.0..=sidebar_w + 2.0,
                                SidebarPosition::Right => (sw - sidebar_w - 2.0)..=(sw - sidebar_w + 2.0),
                            };
                            if self.mouse_x >= *handle_x_range.start() && self.mouse_x <= *handle_x_range.end() 
                                && self.mouse_y > 0.0 
                            {
                                self.is_dragging_sidebar = true;
                                self.drag_start_x = self.mouse_x;
                                self.drag_start_width = self.sidebar_width;
                            }
                            
                            self.window.as_ref().unwrap().request_redraw();
                        } else if state == winit::event::ElementState::Released {
                            self.is_dragging_sidebar = false;
                        }
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        let (line_x, line_y) = match delta {
                            winit::event::MouseScrollDelta::LineDelta(x, y) => (x, y),
                            winit::event::MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
                        };
                        // Determine which panel the mouse is over and scroll it
                        let panel_idx = self.get_panel_at_position(self.mouse_x, self.mouse_y);
                        if let Some(idx) = panel_idx {
                            // Check if mouse is in the file area (not breadcrumb/tab/header/status)
                            let breadcrumb_h = 36.0f32;
                            let tab_h = 32.0f32;
                            let header_h = 28.0f32;
                            let panel_status_h = 24.0f32;
                            let status_h = 30.0f32;
                            let sw = self.window.as_ref().map(|w| w.inner_size().width as f32).unwrap_or(800.0);
                            let sh = self.window.as_ref().map(|w| w.inner_size().height as f32).unwrap_or(600.0);
                            let sidebar_w = if self.sidebar_visible { self.sidebar_width } else { 0.0 };
                            let main_x = if self.sidebar_visible {
                                match self.sidebar_position {
                                    SidebarPosition::Left => sidebar_w + 1.0,
                                    SidebarPosition::Right => 0.0,
                                }
                            } else { 0.0 };
                            let main_w = sw - if self.sidebar_visible { sidebar_w + 1.0 } else { 0.0 };
                            let divider_w = 4.0f32;
                            let (_px, py, _pw, ph) = match self.layout_type {
                                LayoutType::Single => (main_x, 0.0, main_w, sh - status_h),
                                LayoutType::LeftRight => {
                                    let pw = (main_w - divider_w) / 2.0;
                                    if idx == 0 { (main_x, 0.0, pw, sh - status_h) }
                                    else { (main_x + pw + divider_w, 0.0, pw, sh - status_h) }
                                }
                                _ => (main_x, 0.0, main_w, sh - status_h),
                            };
                            let file_area_y = py + breadcrumb_h + tab_h + header_h;
                            let file_area_h = ph - breadcrumb_h - tab_h - header_h - panel_status_h;
                            if self.mouse_y >= file_area_y && self.mouse_y < file_area_y + file_area_h {
                                if self.modifiers.shift_key() {
                                    let h_amount = if line_x.abs() > 0.01 { line_x * 20.0 } else { line_y * 20.0 };
                                    self.panel_scroll_x[idx] -= h_amount;
                                    self.panel_scroll_x[idx] = self.panel_scroll_x[idx].max(0.0_f32);
                                } else {
                                    let scroll_amount = line_y * 20.0;
                                    self.virtual_scroll[idx].handle_scroll(-scroll_amount);
                                    let offset = self.virtual_scroll[idx].sync_to_panel();
                                    if idx == 0 {
                                        self.dual_panel.left_mut().scroll_offset = offset;
                                    } else {
                                        self.dual_panel.right_mut().scroll_offset = offset;
                                    }
                                }
                                self.window.as_ref().unwrap().request_redraw();
                            }
                        }
                    }
                    WindowEvent::KeyboardInput { event, .. } => {
                        if event.state == winit::event::ElementState::Pressed {
                            let key = event.logical_key.clone();
                            let modifiers = self.modifiers;
                            self.handle_key(&key, &modifiers);
                            self.window.as_ref().unwrap().request_redraw();
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        self.render();
                    }
                    _ => {}
                },
                Event::AboutToWait => {
                    // 不再无条件重绘 — 仅在有状态变化时 request_redraw()
                }
                _ => {}
            }
        })?;

        Ok(())
    }

    fn update_hovered_area(&mut self) {
        let _w = self.window.as_ref().unwrap().inner_size().width as f32;
        let h = self.window.as_ref().unwrap().inner_size().height as f32;
        let sidebar_w = self.sidebar_width;
        let breadcrumb_h = 36.0f32;
        let tab_h = 32.0f32;
        let status_h = 30.0f32;

        self.hovered_area = if self.mouse_x < sidebar_w {
            HoveredArea::Sidebar
        } else if self.mouse_y < breadcrumb_h {
            HoveredArea::AddressBar
        } else if self.mouse_y < breadcrumb_h + tab_h {
            HoveredArea::TabBar
        } else if self.mouse_y > h - status_h {
            HoveredArea::StatusBar
        } else {
            HoveredArea::FileList
        };
    }

    fn handle_key(&mut self, key: &winit::keyboard::Key, modifiers: &winit::keyboard::ModifiersState) {
        use winit::keyboard::{Key, NamedKey};

        // 检查是否有路径输入框激活
        let active_panel = if self.dual_panel.is_left_active() { 0 } else { 1 };

        // 路径输入模式
        if self.path_input_active[active_panel] {
            // 路径输入模式
            match key {
                Key::Named(NamedKey::Escape) => {
                    // 取消输入
                    self.path_input_active[active_panel] = false;
                    self.path_input_text[active_panel].clear();
                }
                Key::Named(NamedKey::Enter) => {
                    // 导航到输入的路径
                    let path = self.path_input_text[active_panel].clone();
                    if !path.is_empty() {
                        self.navigate_to(active_panel, &path);
                    }
                    self.path_input_active[active_panel] = false;
                    self.path_input_text[active_panel].clear();
                }
                Key::Named(NamedKey::Backspace) => {
                    // 删除前一个字符
                    let cursor = self.path_input_cursor[active_panel];
                    if cursor > 0 {
                        self.path_input_text[active_panel].remove(cursor - 1);
                        self.path_input_cursor[active_panel] -= 1;
                    }
                }
                Key::Named(NamedKey::Delete) => {
                    // 删除后一个字符
                    let cursor = self.path_input_cursor[active_panel];
                    if cursor < self.path_input_text[active_panel].len() {
                        self.path_input_text[active_panel].remove(cursor);
                    }
                }
                Key::Named(NamedKey::ArrowLeft) => {
                    // 光标左移
                    if self.path_input_cursor[active_panel] > 0 {
                        self.path_input_cursor[active_panel] -= 1;
                    }
                }
                Key::Named(NamedKey::ArrowRight) => {
                    // 光标右移
                    if self.path_input_cursor[active_panel] < self.path_input_text[active_panel].len() {
                        self.path_input_cursor[active_panel] += 1;
                    }
                }
                Key::Named(NamedKey::Home) => {
                    // 光标移到开头
                    self.path_input_cursor[active_panel] = 0;
                }
                Key::Named(NamedKey::End) => {
                    // 光标移到末尾
                    self.path_input_cursor[active_panel] = self.path_input_text[active_panel].len();
                }
                Key::Character(c) => {
                    // 插入字符
                    let cursor = self.path_input_cursor[active_panel];
                    self.path_input_text[active_panel].insert_str(cursor, c.as_str());
                    self.path_input_cursor[active_panel] += c.len();
                }
                _ => {}
            }
            return;
        }

        // 普通模式
        match key {
            Key::Named(NamedKey::Space) => {
                self.preview_visible = !self.preview_visible;
            }
            Key::Named(NamedKey::Escape) => {
                self.preview_visible = false;
                self.search_visible = false;
                self.vim_help_visible = false;
                self.tab_close_confirm = None;
            }
            Key::Named(NamedKey::Tab) => {
                // Tab键切换面板焦点
                self.dual_panel.swap_active();
                // 使用零分配swap交换面板数据
                self.dual_panel.swap_panels();
            }
            Key::Character(c) if c.as_str() == "B" && modifiers.control_key() && modifiers.shift_key() => {
                self.sidebar_visible = !self.sidebar_visible;
            }
            Key::Character(c) if c.as_str() == "f" && modifiers.control_key() => {
                self.search_visible = !self.search_visible;
            }
            Key::Character(c) if c.as_str() == "t" && modifiers.control_key() => {
                // Ctrl+T 切换主题
                if let Some(gpu) = &mut self.gpu {
                    gpu.toggle_theme();
                }
            }
            Key::Character(c) if c.as_str() == "?" => {
                self.vim_help_visible = !self.vim_help_visible;
            }
            // Backspace: 返回上一级目录
            Key::Named(NamedKey::Backspace) => {
                self.navigate_up(active_panel);
            }
            // Enter: 进入选中的文件夹
            Key::Named(NamedKey::Enter) => {
                let file_path = if active_panel == 0 {
                    self.dual_panel.left().selected_indices.first()
                        .and_then(|&idx| {
                            let files = &self.dual_panel.left().files;
                            if idx < files.len() && files[idx].is_dir {
                                Some(files[idx].path.clone())
                            } else {
                                None
                            }
                        })
                } else {
                    self.dual_panel.right().selected_indices.first()
                        .and_then(|&idx| {
                            let files = &self.dual_panel.right().files;
                            if idx < files.len() && files[idx].is_dir {
                                Some(files[idx].path.clone())
                            } else {
                                None
                            }
                        })
                };
                if let Some(path) = file_path {
                    self.navigate_to(active_panel, &path);
                }
            }
            _ => {}
        }
    }

    fn handle_click(&mut self, button: winit::event::MouseButton) {
        if button != winit::event::MouseButton::Left {
            return;
        }

        let sw = self.window.as_ref().unwrap().inner_size().width as f32;
        let sh = self.window.as_ref().unwrap().inner_size().height as f32;
        let status_h = 30.0f32;
        let status_y = sh - status_h;

        // Tab close confirmation dialog
        if let Some((panel_idx, tab_idx)) = self.tab_close_confirm {
            let dialog_w = 320.0;
            let dialog_h = 140.0;
            let dialog_x = (sw - dialog_w) / 2.0;
            let dialog_y = (sh - dialog_h) / 2.0;

            let cancel_x = dialog_x + dialog_w - 180.0;
            let cancel_y = dialog_y + dialog_h - 44.0;
            let btn_w = 76.0;
            let btn_h = 32.0;

            // 取消按钮
            if self.mouse_x >= cancel_x && self.mouse_x < cancel_x + btn_w
                && self.mouse_y >= cancel_y && self.mouse_y < cancel_y + btn_h {
                self.tab_close_confirm = None;
                return;
            }

            // 确定按钮
            let confirm_x = dialog_x + dialog_w - 92.0;
            if self.mouse_x >= confirm_x && self.mouse_x < confirm_x + btn_w
                && self.mouse_y >= cancel_y && self.mouse_y < cancel_y + btn_h {
                // 关闭标签
                if self.panel_tabs[panel_idx].len() > 1 {
                    self.panel_tabs[panel_idx].remove(tab_idx);
                    // 调整活跃标签索引
                    if self.active_tab_idx[panel_idx] >= self.panel_tabs[panel_idx].len() {
                        self.active_tab_idx[panel_idx] = self.panel_tabs[panel_idx].len() - 1;
                    }
                    // 导航到新活跃标签的路径
                    let tab_path = self.panel_tabs[panel_idx][self.active_tab_idx[panel_idx]].path.clone();
                    self.navigate_to(panel_idx, &tab_path);
                }
                self.tab_close_confirm = None;
                return;
            }

            // 点击对话框外部取消
            if !(self.mouse_x >= dialog_x && self.mouse_x < dialog_x + dialog_w
                && self.mouse_y >= dialog_y && self.mouse_y < dialog_y + dialog_h) {
                self.tab_close_confirm = None;
                return;
            }
            return;
        }

        // Status bar click area: y > status_y
        if self.mouse_y > status_y {
            let mode_x = 12.0;

            // Mode toggle (panel/cascade)
            if self.mouse_x >= mode_x && self.mouse_x < mode_x + 20.0 {
                self.panel_mode = PanelMode::Panels;
            } else if self.mouse_x >= mode_x + 22.0 && self.mouse_x < mode_x + 42.0 {
                self.panel_mode = PanelMode::Cascade;
            }

            // Sidebar position toggle
            let sidebar_pos_x = mode_x + 56.0;
            if self.mouse_x >= sidebar_pos_x && self.mouse_x < sidebar_pos_x + 28.0 {
                self.sidebar_position = SidebarPosition::Left;
            } else if self.mouse_x >= sidebar_pos_x + 30.0 && self.mouse_x < sidebar_pos_x + 58.0 {
                self.sidebar_position = SidebarPosition::Right;
            }

            // Space toggle
            let space_x = sidebar_pos_x + 70.0;
            let spaces = ["default", "work", "dev"];
            let mut sx = space_x;
            for id in spaces.iter() {
                let label = match *id {
                    "default" => "[HOME] 默认",
                    "work" => "[WORK] Work",
                    _ => "[DEV] Dev",
                };
                let btn_w = self.measure_text_width(label) + 16.0;
                if self.mouse_x >= sx && self.mouse_x < sx + btn_w {
                    self.active_space = *id;
                }
                sx += btn_w + 2.0;
            }

            // Layout toggle
            let layout_x = sw - 160.0;
            for i in 1..=7u8 {
                let bx = layout_x + (i - 1) as f32 * 22.0;
                if self.mouse_x >= bx && self.mouse_x < bx + 20.0 {
                    self.layout_type = match i {
                        1 => LayoutType::Single,
                        2 => LayoutType::LeftRight,
                        3 => LayoutType::TopBottom,
                        4 => LayoutType::LeftMidRight,
                        5 => LayoutType::Top2Bottom1,
                        6 => LayoutType::Top1Bottom2,
                        7 => LayoutType::FourGrid,
                        _ => LayoutType::LeftRight,
                    };
                }
            }
        }

        // Sidebar space section click
        let sidebar_x = if self.sidebar_visible {
            match self.sidebar_position {
                SidebarPosition::Left => 0.0,
                SidebarPosition::Right => sw - self.sidebar_width - 1.0,
            }
        } else {
            -1.0 // No sidebar
        };
        if self.sidebar_visible && self.mouse_x >= sidebar_x && self.mouse_x < sidebar_x + self.sidebar_width {
            // Calculate space section Y position
            let mut sy = 12.0f32 + 24.0; // 此电脑 title + 3 disks * (row_h + 28) 
            sy += 3.0 * 28.0 + 8.0; // disks
            sy += 24.0 + 4.0 * 28.0 + 8.0; // 标签 + bookmarks + add
            sy += 24.0 + 4.0 * 28.0 + 8.0; // 最近访问 + recents
            sy += 24.0; // 空间 title

            // Space items
            let spaces = ["default", "work", "dev"];
            for id in spaces.iter() {
                if self.mouse_y >= sy && self.mouse_y < sy + 28.0 {
                    self.active_space = *id;
                }
                sy += 28.0;
            }
        }

        // Cascade mode click
        if self.panel_mode == PanelMode::Cascade {
            let main_x = if self.sidebar_visible {
                match self.sidebar_position {
                    SidebarPosition::Left => self.sidebar_width + 1.0,
                    SidebarPosition::Right => 0.0,
                }
            } else {
                0.0
            };
            let col_w = 200.0f32;
            let row_h = 28.0f32;
            let cascade_columns = [3, 3, 2, 3]; // items per column
            for (col_idx, &item_count) in cascade_columns.iter().enumerate() {
                let col_x = main_x + col_idx as f32 * (col_w + 1.0);
                if self.mouse_x >= col_x && self.mouse_x < col_x + col_w && self.mouse_y >= 28.0 {
                    let item_idx = ((self.mouse_y - 28.0) / row_h) as usize;
                    if item_idx < item_count {
                        self.cascade_selected[col_idx] = item_idx;
                        // Reset selections in deeper columns
                        for deeper in (col_idx + 1)..4 {
                            self.cascade_selected[deeper] = 0;
                        }
                    }
                }
            }
        }

        // Breadcrumb click handling
        if self.panel_mode == PanelMode::Panels {
            let breadcrumb_h = 36.0f32;
            let status_h = 30.0f32;
            let divider_w = 4.0f32;

            let (main_x, main_w) = if self.sidebar_visible {
                match self.sidebar_position {
                    SidebarPosition::Left => (self.sidebar_width + 1.0, sw - self.sidebar_width - 1.0),
                    SidebarPosition::Right => (0.0, sw - self.sidebar_width - 1.0),
                }
            } else {
                (0.0, sw)
            };

            let panel_count = match self.layout_type {
                LayoutType::Single => 1,
                LayoutType::LeftRight | LayoutType::TopBottom => 2,
                LayoutType::LeftMidRight | LayoutType::Top2Bottom1 | LayoutType::Top1Bottom2 => 3,
                LayoutType::FourGrid => 4,
            };

            for panel_idx in 0..panel_count {
                let (px, py, pw, _ph) = match self.layout_type {
                    LayoutType::Single => (main_x, 0.0, main_w, sh - status_h),
                    LayoutType::LeftRight => {
                        let pw = (main_w - divider_w) / 2.0;
                        if panel_idx == 0 { (main_x, 0.0, pw, sh - status_h) } else { (main_x + pw + divider_w, 0.0, pw, sh - status_h) }
                    }
                    LayoutType::TopBottom => {
                        let ph = (sh - status_h - divider_w) / 2.0;
                        if panel_idx == 0 { (main_x, 0.0, main_w, ph) } else { (main_x, ph + divider_w, main_w, ph) }
                    }
                    LayoutType::LeftMidRight => {
                        let pw = (main_w - divider_w * 2.0) / 3.0;
                        match panel_idx {
                            0 => (main_x, 0.0, pw, sh - status_h),
                            1 => (main_x + pw + divider_w, 0.0, pw, sh - status_h),
                            _ => (main_x + (pw + divider_w) * 2.0, 0.0, pw, sh - status_h),
                        }
                    }
                    _ => (main_x, 0.0, main_w, sh - status_h),
                };

                // 检查是否点击了面包屑区域
                if self.mouse_y >= py && self.mouse_y < py + breadcrumb_h
                    && self.mouse_x >= px && self.mouse_x < px + pw {

                    let is_path_input = self.path_input_active[panel_idx];

                    if is_path_input {
                        // 路径输入模式：检查是否点击了输入框外部（取消输入）
                        let input_x = px + 8.0;
                        let input_w = pw - 16.0;
                        let input_y = py + 6.0;
                        let input_h = 24.0;

                        if !(self.mouse_x >= input_x && self.mouse_x < input_x + input_w
                            && self.mouse_y >= input_y && self.mouse_y < input_y + input_h) {
                            // 点击外部，取消输入
                            self.path_input_active[panel_idx] = false;
                            self.path_input_text[panel_idx].clear();
                        }
                    } else {
                        // 面包屑模式：双击切换到输入模式
                        let now = std::time::Instant::now();
                        let is_double_click = panel_idx == self.last_click_panel
                            && now.duration_since(self.last_click_time).as_millis() < 300;
                        self.last_click_time = now;
                        self.last_click_panel = panel_idx;

                        if is_double_click {
                            // 双击：切换到路径输入模式
                            let panel_path = if panel_idx == 0 {
                                self.dual_panel.left().path.clone()
                            } else {
                                self.dual_panel.right().path.clone()
                            };
                            self.path_input_active[panel_idx] = true;
                            self.path_input_text[panel_idx] = panel_path;
                            self.path_input_cursor[panel_idx] = self.path_input_text[panel_idx].len();
                        } else {
                            // 单击：检查是否点击了某个面包屑段
                            let panel_path = if panel_idx == 0 {
                                self.dual_panel.left().path.clone()
                            } else {
                                self.dual_panel.right().path.clone()
                            };
                            let path_components: Vec<std::path::Component> = std::path::Path::new(&panel_path)
                                .components()
                                .collect();
                            let crumbs: Vec<String> = path_components
                                .iter()
                                .map(|c| c.as_os_str().to_string_lossy().to_string())
                                .filter(|s| !s.is_empty())
                                .collect();

                            // 计算每个面包屑的位置并检查点击
                            let mut cx = px + 12.0;
for (i, _crumb) in crumbs.iter().enumerate() {
                                let text_w = 80.0; // 估算宽度
                                let separator_w = if i < crumbs.len() - 1 { 24.0 } else { 0.0 };
                                let total_w = text_w + separator_w;

                                if self.mouse_x >= cx && self.mouse_x < cx + total_w {
                                    // 点击了这个面包屑段，导航到对应路径
                                    let target_path: String = crumbs[..=i].join(
                                        if cfg!(target_os = "windows") { "\\" } else { "/" }
                                    );
                                    // 如果是Windows，需要加上盘符后的反斜杠
                                    let target_path = if cfg!(target_os = "windows") && i == 0 {
                                        format!("{}\\", target_path)
                                    } else {
                                        target_path
                                    };
                                    self.navigate_to(panel_idx, &target_path);
                                    break;
                                }
                                cx += total_w;
                            }
                        }
                    }
                    break;
                }
            }
        }

        // Tab bar click handling
        if self.panel_mode == PanelMode::Panels {
            let breadcrumb_h = 36.0f32;
            let tab_h = 32.0f32;
            let status_h = 30.0f32;
            let divider_w = 4.0f32;

            let (main_x, main_w) = if self.sidebar_visible {
                match self.sidebar_position {
                    SidebarPosition::Left => (self.sidebar_width + 1.0, sw - self.sidebar_width - 1.0),
                    SidebarPosition::Right => (0.0, sw - self.sidebar_width - 1.0),
                }
            } else {
                (0.0, sw)
            };

            let panel_count = match self.layout_type {
                LayoutType::Single => 1,
                LayoutType::LeftRight | LayoutType::TopBottom => 2,
                LayoutType::LeftMidRight | LayoutType::Top2Bottom1 | LayoutType::Top1Bottom2 => 3,
                LayoutType::FourGrid => 4,
            };

            for panel_idx in 0..panel_count {
                let (px, py, pw, _ph) = match self.layout_type {
                    LayoutType::Single => (main_x, 0.0, main_w, sh - status_h),
                    LayoutType::LeftRight => {
                        let pw = (main_w - divider_w) / 2.0;
                        if panel_idx == 0 { (main_x, 0.0, pw, sh - status_h) } else { (main_x + pw + divider_w, 0.0, pw, sh - status_h) }
                    }
                    LayoutType::TopBottom => {
                        let ph = (sh - status_h - divider_w) / 2.0;
                        if panel_idx == 0 { (main_x, 0.0, main_w, ph) } else { (main_x, ph + divider_w, main_w, ph) }
                    }
                    LayoutType::LeftMidRight => {
                        let pw = (main_w - divider_w * 2.0) / 3.0;
                        match panel_idx {
                            0 => (main_x, 0.0, pw, sh - status_h),
                            1 => (main_x + pw + divider_w, 0.0, pw, sh - status_h),
                            _ => (main_x + (pw + divider_w) * 2.0, 0.0, pw, sh - status_h),
                        }
                    }
                    _ => (main_x, 0.0, main_w, sh - status_h),
                };

                let tab_y = py + breadcrumb_h;

                // 检查是否点击了标签栏区域
                if self.mouse_y >= tab_y && self.mouse_y < tab_y + tab_h
                    && self.mouse_x >= px && self.mouse_x < px + pw {

                    let now = std::time::Instant::now();
                    let is_double_click = panel_idx == self.last_click_panel
                        && now.duration_since(self.last_click_time).as_millis() < 300;
                    self.last_click_time = now;
                    self.last_click_panel = panel_idx;

                    if is_double_click {
                        // 双击标签栏：检查是否点击了某个标签
                        let positions = &self.tab_positions[panel_idx];
                        let mut clicked_tab_idx = None;

                        for (i, (tx, tab_w)) in positions.iter().enumerate() {
                            if self.mouse_x >= *tx && self.mouse_x < *tx + *tab_w {
                                clicked_tab_idx = Some(i);
                                break;
                            }
                        }

                        if let Some(tab_idx) = clicked_tab_idx {
                            // 双击标签：弹出确认关闭对话框
                            if self.panel_tabs[panel_idx].len() > 1 {
                                self.tab_close_confirm = Some((panel_idx, tab_idx));
                            }
                        } else {
                            // 双击空白处：创建新标签
                            let current_path = if panel_idx == 0 {
                                self.dual_panel.left().path.clone()
                            } else {
                                self.dual_panel.right().path.clone()
                            };
                            let dir_name = std::path::Path::new(&current_path)
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "New Tab".to_string());
                            self.panel_tabs[panel_idx].push(TabInfo {
                                name: dir_name,
                                path: current_path,
                            });
                            self.active_tab_idx[panel_idx] = self.panel_tabs[panel_idx].len() - 1;
                        }
                    } else {
                        // 单击标签：切换到该标签
                        let positions = &self.tab_positions[panel_idx];
                        let mut clicked_tab_idx = None;

                        for (i, (tx, tab_w)) in positions.iter().enumerate() {
                            if self.mouse_x >= *tx && self.mouse_x < *tx + *tab_w {
                                clicked_tab_idx = Some(i);
                                break;
                            }
                        }

                        if let Some(tab_idx) = clicked_tab_idx {
                            self.active_tab_idx[panel_idx] = tab_idx;
                            // 导航到标签对应的路径
                            let tab_path = self.panel_tabs[panel_idx][tab_idx].path.clone();
                            self.navigate_to(panel_idx, &tab_path);
                        }
                    }
                    break;
                }
            }
        }

        // File list click handling (panels mode)
        if self.panel_mode == PanelMode::Panels {
            let breadcrumb_h = 36.0f32;
            let tab_h = 32.0f32;
            let header_h = 28.0f32;
            let row_h = 28.0f32;
            let panel_status_h = 24.0f32;
            let status_h = 30.0f32;
            let divider_w = 4.0f32;

            let (main_x, main_w) = if self.sidebar_visible {
                match self.sidebar_position {
                    SidebarPosition::Left => (self.sidebar_width + 1.0, sw - self.sidebar_width - 1.0),
                    SidebarPosition::Right => (0.0, sw - self.sidebar_width - 1.0),
                }
            } else {
                (0.0, sw)
            };

            let panel_count = match self.layout_type {
                LayoutType::Single => 1,
                LayoutType::LeftRight | LayoutType::TopBottom => 2,
                LayoutType::LeftMidRight | LayoutType::Top2Bottom1 | LayoutType::Top1Bottom2 => 3,
                LayoutType::FourGrid => 4,
            };

            for panel_idx in 0..panel_count {
                let (px, py, pw, ph) = match self.layout_type {
                    LayoutType::Single => (main_x, 0.0, main_w, sh - status_h),
                    LayoutType::LeftRight => {
                        let pw = (main_w - divider_w) / 2.0;
                        if panel_idx == 0 { (main_x, 0.0, pw, sh - status_h) } else { (main_x + pw + divider_w, 0.0, pw, sh - status_h) }
                    }
                    LayoutType::TopBottom => {
                        let ph = (sh - status_h - divider_w) / 2.0;
                        if panel_idx == 0 { (main_x, 0.0, main_w, ph) } else { (main_x, ph + divider_w, main_w, ph) }
                    }
                    LayoutType::LeftMidRight => {
                        let pw = (main_w - divider_w * 2.0) / 3.0;
                        match panel_idx {
                            0 => (main_x, 0.0, pw, sh - status_h),
                            1 => (main_x + pw + divider_w, 0.0, pw, sh - status_h),
                            _ => (main_x + (pw + divider_w) * 2.0, 0.0, pw, sh - status_h),
                        }
                    }
                    _ => (main_x, 0.0, main_w, sh - status_h),
                };

                if self.mouse_x >= px && self.mouse_x < px + pw && self.mouse_y >= py && self.mouse_y < py + ph {
                    let file_area_y = py + breadcrumb_h + tab_h + header_h;
                    let file_area_h = ph - breadcrumb_h - tab_h - header_h - panel_status_h;

                    // click log removed for cleanliness

if self.mouse_y >= file_area_y && self.mouse_y < file_area_y + file_area_h {
                        // 先获取需要的数据，避免借用冲突
                        let (file_count, scroll_y, file_path, file_is_dir) = {
                            let panel_snapshot = if panel_idx == 0 { self.dual_panel.left() } else { self.dual_panel.right() };
                            let file_count = panel_snapshot.files.len();
                            let scroll_y = self.virtual_scroll[panel_idx.min(3)].scroll_offset();
                            let row_idx = ((self.mouse_y - file_area_y + scroll_y) / row_h) as usize;
                            let (fp, fid) = if row_idx < file_count {
                                (panel_snapshot.files[row_idx].path.clone(), panel_snapshot.files[row_idx].is_dir)
                            } else {
                                (String::new(), false)
                            };
                            (file_count, scroll_y, fp, fid)
                        };
                        let row_idx = ((self.mouse_y - file_area_y + scroll_y) / row_h) as usize;

                        // click row log removed

                        if row_idx < file_count {
                            let should_swap = (panel_idx == 0 && !self.dual_panel.is_left_active())
                                || (panel_idx == 1 && self.dual_panel.is_left_active());
                            if should_swap {
                                self.dual_panel.swap_active();
                            }

                            let ctrl_held = self.modifiers.control_key();
                            let panel = self.dual_panel.active_mut();

                            if ctrl_held {
                                if panel.selected_indices.contains(&row_idx) {
                                    panel.selected_indices.retain(|&i| i != row_idx);
                                } else {
                                    panel.selected_indices.push(row_idx);
                                }
                            } else {
                                panel.selected_indices.clear();
                                panel.selected_indices.push(row_idx);
                            }

                            // 双击检测: 同一行 + 300ms 内 → 导航进入文件夹
                            let now = std::time::Instant::now();
                            let is_double_click = row_idx == self.last_click_idx
                                && panel_idx == self.last_click_panel
                                && now.duration_since(self.last_click_time).as_millis() < 300;
                            self.last_click_time = now;
                            self.last_click_panel = panel_idx;
                            self.last_click_idx = row_idx;

                            if is_double_click && file_is_dir && !file_path.is_empty() {
                                self.navigate_to(panel_idx, &file_path);
                            }
                        }
                        break;
                    }
                }
            }
        }
    }

    fn measure_text_width(&self, text: &str) -> f32 {
        if let Some(gpu) = &self.gpu {
            gpu.measure_text(text)
        } else {
            text.len() as f32 * 8.0
        }
    }

    fn render(&mut self) {
        // 缓存数据，避免借用冲突
        let left_name = self.current_dir_name(0);
        let right_name = self.current_dir_name(1);
        let left_count = self.file_count(0);
        let right_count = self.file_count(1);
        let active_panel = if self.dual_panel.is_left_active() { 0 } else { 1 };
        let total_files = left_count + right_count;
        let selected = self.selected_count(active_panel);
        let _current_path = if active_panel == 0 {
            self.dual_panel.left().path.clone()
        } else {
            self.dual_panel.right().path.clone()
        };
        let left_files: Vec<(String, bool)> = self.dual_panel.left().files.iter().take(10).map(|f| (f.name.clone(), f.is_dir)).collect();
        let right_files: Vec<(String, bool)> = self.dual_panel.right().files.iter().take(10).map(|f| (f.name.clone(), f.is_dir)).collect();
        let left_path = self.dual_panel.left().path.clone();
        let right_path = self.dual_panel.right().path.clone();
        let active_panel_path = if active_panel == 0 { left_path.clone() } else { right_path.clone() };

        if let Some(gpu) = &mut self.gpu {
                if let Some(frame) = gpu.begin_frame() {
                    let view = frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());

                    let mut encoder = gpu
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Zero Explorer Encoder"),
                        });

                    // 使用主题颜色
let colors = gpu.theme_colors();
                    let bg_base = colors.bg_primary;
                let bg_secondary = colors.bg_secondary;
                let bg_tertiary = colors.bg_tertiary;
                let border = colors.border;
                let text_primary = colors.fg_primary;
                let text_secondary = colors.fg_secondary;
                let text_tertiary = colors.fg_disabled;
                let primary = colors.accent;
                let primary_light = colors.bg_selected;
                let white = [1.0, 1.0, 1.0, 1.0];
                let success = colors.success;
                let success_light = colors.bg_selected;
                let warning = colors.warning;
                let warning_light = colors.bg_selected;

                gpu.clear(&mut encoder, &view, wgpu::Color { 
                    r: bg_base[0] as f64, 
                    g: bg_base[1] as f64, 
                    b: bg_base[2] as f64, 
                    a: bg_base[3] as f64 
                });

                // 处理图标响应并上传到纹理图集
                gpu.process_icon_responses();

                let sw = gpu.surface_config.width as f32;
                let sh = gpu.surface_config.height as f32;
                let sidebar_w = self.sidebar_width;
                let breadcrumb_h = 36.0f32;
                let tab_h = 32.0f32;
                let status_h = 30.0f32;
                let row_h = 28.0f32;
                let header_h = 28.0f32;
                let panel_status_h = 24.0f32;

                // === SIDEBAR ===
                if self.sidebar_visible {
                    let sidebar_x = match self.sidebar_position {
                        SidebarPosition::Left => 0.0,
                        SidebarPosition::Right => sw - sidebar_w - 1.0,
                    };
                    gpu.draw_rect_simple(&mut encoder, &view, sidebar_x, 0.0, sidebar_w, sh, bg_secondary, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, sidebar_x + sidebar_w, 0.0, 1.0, sh, border, sw, sh);

                    // Sidebar resize handle
                    let handle_x = sidebar_x + sidebar_w - 2.0;
                    let handle_color = if self.is_dragging_sidebar || 
                        (self.mouse_x >= handle_x && self.mouse_x <= handle_x + 4.0) {
                        primary
                    } else {
                        [0.0, 0.0, 0.0, 0.0] // transparent
                    };
                    gpu.draw_rect_simple(&mut encoder, &view, handle_x, 0.0, 4.0, sh, handle_color, sw, sh);

                    // Sidebar resize tooltip
                    if self.is_dragging_sidebar || (self.mouse_x >= handle_x && self.mouse_x <= handle_x + 4.0) {
                        let tooltip_text = format!("{}px", sidebar_w as u32);
                        let tooltip_w_px = gpu.measure_text(&tooltip_text) + 16.0;
                        let tooltip_x = sidebar_x + sidebar_w - tooltip_w_px / 2.0;
                        let tooltip_y = sh / 2.0 - 10.0;
                        gpu.draw_rect_simple(&mut encoder, &view, tooltip_x, tooltip_y, tooltip_w_px, 20.0, text_primary, sw, sh);
                        gpu.draw_text_simple(&mut encoder, &view, &tooltip_text, tooltip_x + 8.0, Self::text_y_centered(gpu, tooltip_y, 20.0), white, sw, sh);
                    }

                    let mut sy = 12.0f32;

                    // Section: 当前面板
                    gpu.draw_text_simple(&mut encoder, &view, "当前面板 ▾", sidebar_x + 12.0, Self::text_y_centered(gpu, sy, 20.0), text_secondary, sw, sh);
                    sy += 24.0;

                    // 显示左面板路径
                    let left_item_bg = if self.mouse_y >= sy && self.mouse_y < sy + row_h && self.mouse_x >= sidebar_x && self.mouse_x < sidebar_x + sidebar_w {
                        bg_tertiary
                    } else {
                        bg_secondary
                    };
                    gpu.draw_rect_simple(&mut encoder, &view, sidebar_x + 4.0, sy, sidebar_w - 8.0, row_h, left_item_bg, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, &format!("◧ {} ({})", left_name, left_count), sidebar_x + 12.0, Self::text_y_centered(gpu, sy, row_h), primary, sw, sh);
                    sy += row_h;

                    // 显示右面板路径
                    let right_item_bg = if self.mouse_y >= sy && self.mouse_y < sy + row_h && self.mouse_x >= sidebar_x && self.mouse_x < sidebar_x + sidebar_w {
                        bg_tertiary
                    } else {
                        bg_secondary
                    };
                    gpu.draw_rect_simple(&mut encoder, &view, sidebar_x + 4.0, sy, sidebar_w - 8.0, row_h, right_item_bg, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, &format!("◨ {} ({})", right_name, right_count), sidebar_x + 12.0, Self::text_y_centered(gpu, sy, row_h), primary, sw, sh);
                    sy += row_h + 8.0;

                    // Section: 文件列表 (当前面板的文件)
                    gpu.draw_text_simple(&mut encoder, &view, "文件列表", sidebar_x + 12.0, Self::text_y_centered(gpu, sy, 20.0), text_secondary, sw, sh);
                    sy += 24.0;
                    // 使用缓存的文件列表
                    let active_files = if active_panel == 0 { &left_files } else { &right_files };
                    for (name, is_dir) in active_files.iter() {
                        let file_icon = FileIcon::from_path(name);
                        let icon_color = if *is_dir { [1.0, 0.8, 0.0, 1.0] } else { file_icon.icon_color() };
                        let icon_y = sy + (row_h - 24.0) / 2.0;
                        gpu.draw_file_icon(&mut encoder, &view, file_icon, icon_color, name, sidebar_x + 12.0, icon_y, 24.0, sw, sh);
                        gpu.draw_text_simple(&mut encoder, &view, name, sidebar_x + 42.0, Self::text_y_centered(gpu, sy, row_h), text_primary, sw, sh);
                        sy += row_h;
                    }
                    let total_active = if active_panel == 0 { left_count } else { right_count };
                    if total_active > 10 {
                        gpu.draw_text_simple(&mut encoder, &view, &format!("... 还有 {} 项", total_active - 10), sidebar_x + 12.0, Self::text_y_centered(gpu, sy, row_h), text_tertiary, sw, sh);
                        sy += row_h;
                    }

                    sy += 8.0;
                    // Section: 空间
                    gpu.draw_text_simple(&mut encoder, &view, "空间", sidebar_x + 12.0, Self::text_y_centered(gpu, sy, 20.0), text_secondary, sw, sh);
                    sy += 24.0;
                    let spaces = [
                        ("default", "[HOME] 默认", primary, primary_light),
                        ("work", "[WORK] Work", success, success_light),
                        ("dev", "[DEV] Dev", warning, warning_light),
                    ];
                    for (id, label, color, light_color) in spaces.iter() {
                        let is_active = self.active_space == *id;
                        let item_bg = if is_active { *light_color } else { bg_secondary };
                        let item_text = if is_active { *color } else { text_primary };
                        gpu.draw_rect_simple(&mut encoder, &view, sidebar_x + 4.0, sy, sidebar_w - 8.0, row_h, item_bg, sw, sh);
                        gpu.draw_text_simple(&mut encoder, &view, label, sidebar_x + 12.0, Self::text_y_centered(gpu, sy, row_h), item_text, sw, sh);
                        sy += row_h;
                    }
                    gpu.draw_text_simple(&mut encoder, &view, "+ 新建空间", sidebar_x + 12.0, Self::text_y_centered(gpu, sy, row_h), primary, sw, sh);
                }

                // === MAIN CONTENT AREA ===
            let (main_x, main_w) = if self.sidebar_visible {
                    match self.sidebar_position {
                        SidebarPosition::Left => (sidebar_w + 1.0, sw - sidebar_w - 1.0),
                        SidebarPosition::Right => (0.0, sw - sidebar_w - 1.0),
                    }
                } else {
                    (0.0, sw)
                };

                // Calculate panel dimensions based on layout_type
                let divider_w = 4.0f32;
                let panel_count = match self.layout_type {
                    LayoutType::Single => 1,
                    LayoutType::LeftRight => 2,
                    LayoutType::TopBottom => 2,
                    LayoutType::LeftMidRight => 3,
                    LayoutType::Top2Bottom1 => 3,
                    LayoutType::Top1Bottom2 => 3,
                    LayoutType::FourGrid => 4,
                };

                // Helper closure to get panel position (x, y, w, h)
                let get_panel_rect = |idx: usize| -> (f32, f32, f32, f32) {
                    match self.layout_type {
                        LayoutType::Single => {
                            (main_x, 0.0, main_w, sh - status_h)
                        }
                        LayoutType::LeftRight => {
                            let pw = (main_w - divider_w) / 2.0;
                            if idx == 0 {
                                (main_x, 0.0, pw, sh - status_h)
                            } else {
                                (main_x + pw + divider_w, 0.0, pw, sh - status_h)
                            }
                        }
                        LayoutType::TopBottom => {
                            let ph = (sh - status_h - divider_w) / 2.0;
                            if idx == 0 {
                                (main_x, 0.0, main_w, ph)
                            } else {
                                (main_x, ph + divider_w, main_w, ph)
                            }
                        }
                        LayoutType::LeftMidRight => {
                            let pw = (main_w - divider_w * 2.0) / 3.0;
                            (main_x + idx as f32 * (pw + divider_w), 0.0, pw, sh - status_h)
                        }
                        LayoutType::Top2Bottom1 => {
                            let pw = (main_w - divider_w) / 2.0;
                            let ph = (sh - status_h - divider_w) / 2.0;
                            match idx {
                                0 => (main_x, 0.0, pw, ph),
                                1 => (main_x + pw + divider_w, 0.0, pw, ph),
                                _ => (main_x, ph + divider_w, main_w, ph),
                            }
                        }
                        LayoutType::Top1Bottom2 => {
                            let pw = (main_w - divider_w) / 2.0;
                            let ph = (sh - status_h - divider_w) / 2.0;
                            match idx {
                                0 => (main_x, 0.0, main_w, ph),
                                _ => (main_x + (idx as f32 - 1.0) * (pw + divider_w), ph + divider_w, pw, ph),
                            }
                        }
                        LayoutType::FourGrid => {
                            let pw = (main_w - divider_w) / 2.0;
                            let ph = (sh - status_h - divider_w) / 2.0;
                            let row = idx / 2;
                            let col = idx % 2;
                            (main_x + col as f32 * (pw + divider_w), row as f32 * (ph + divider_w), pw, ph)
                        }
                    }
                };

                // Render content based on mode
                if self.panel_mode == PanelMode::Panels {
                    // Render panels
                    for panel_idx in 0..panel_count {
                    let (panel_x, panel_y, panel_w, panel_h) = get_panel_rect(panel_idx);

                    // Panel background
                    gpu.draw_rect_simple(&mut encoder, &view, panel_x, panel_y, panel_w, panel_h, bg_base, sw, sh);

                    // Panel border
                    if panel_idx > 0 {
                        gpu.draw_rect_simple(&mut encoder, &view, panel_x - divider_w, panel_y, divider_w, panel_h, border, sw, sh);
                    }

                    // Breadcrumb bar
                    gpu.draw_rect_simple(&mut encoder, &view, panel_x, panel_y, panel_w, breadcrumb_h, bg_base, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, panel_x, panel_y + breadcrumb_h - 1.0, panel_w, 1.0, border, sw, sh);

                    // Breadcrumb items (从当前面板路径生成)
                    let panel_path = if panel_idx == 0 {
                        self.dual_panel.left().path.clone()
                    } else {
                        self.dual_panel.right().path.clone()
                    };
                    let path_components: Vec<std::path::Component> = std::path::Path::new(&panel_path)
                        .components()
                        .collect();
                    let crumbs: Vec<String> = path_components
                        .iter()
                        .map(|c| c.as_os_str().to_string_lossy().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();

                    let is_path_input = self.path_input_active[panel_idx];

                    if is_path_input {
                        // 路径输入模式：显示可编辑的输入框
                        let input_x = panel_x + 8.0;
                        let input_w = panel_w - 16.0;
                        let input_y = panel_y + 6.0;
                        let input_h = 24.0;

                        // 输入框背景
                        gpu.draw_rect_simple(&mut encoder, &view, input_x, input_y, input_w, input_h, bg_base, sw, sh);
                        gpu.draw_rect_simple(&mut encoder, &view, input_x, input_y, input_w, input_h, primary, sw, sh);

                        // 显示输入文本
                        let input_text = &self.path_input_text[panel_idx];
                        let display_text = if input_text.is_empty() { &panel_path } else { input_text };
                        gpu.draw_text_simple(&mut encoder, &view, display_text, input_x + 8.0, Self::text_y_centered(gpu, input_y, input_h), text_primary, sw, sh);

                        // 光标
                        let cursor_x = input_x + 8.0 + gpu.measure_text(&display_text[..self.path_input_cursor[panel_idx].min(display_text.len())]);
                        gpu.draw_rect_simple(&mut encoder, &view, cursor_x, input_y + 4.0, 2.0, input_h - 8.0, primary, sw, sh);
                    } else {
                        // 面包屑模式：显示可点击的路径段
                        let mut cx = panel_x + 12.0;
                        for (i, crumb) in crumbs.iter().enumerate() {
                            let is_last = i == crumbs.len() - 1;
                            let color = if is_last { text_primary } else { text_secondary };

                            // 计算这段的宽度
                            let text_w = gpu.measure_text(crumb);
                            let separator_w = if !is_last { gpu.measure_text(" › ") } else { 0.0 };
                            let total_w = text_w + separator_w;

                            // 检测hover
                            let is_hover = self.mouse_x >= cx && self.mouse_x < cx + total_w
                                && self.mouse_y >= panel_y && self.mouse_y < panel_y + breadcrumb_h;

                            // hover时显示背景
                            if is_hover {
                                gpu.draw_rect_simple(&mut encoder, &view, cx - 2.0, panel_y + 4.0, total_w + 4.0, breadcrumb_h - 8.0, bg_tertiary, sw, sh);
                            }

                            gpu.draw_text_simple(&mut encoder, &view, crumb, cx, Self::text_y_centered(gpu, panel_y, breadcrumb_h), color, sw, sh);
                            cx += text_w;

                            if !is_last {
                                gpu.draw_text_simple(&mut encoder, &view, " › ", cx, Self::text_y_centered(gpu, panel_y, breadcrumb_h), text_tertiary, sw, sh);
                                cx += separator_w;
                            }
                        }
                    }

                    // Tab bar
                    let tab_y = panel_y + breadcrumb_h;
                    gpu.draw_rect_simple(&mut encoder, &view, panel_x, tab_y, panel_w, tab_h, bg_tertiary, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, panel_x, tab_y + tab_h - 1.0, panel_w, 1.0, border, sw, sh);

                    // Tabs (使用panel_tabs)
                    let tabs = &self.panel_tabs[panel_idx];
                    let active_tab = self.active_tab_idx[panel_idx];
                    let mut tx = panel_x + 4.0;
                    let mut positions = Vec::new();
                    for (i, tab) in tabs.iter().enumerate() {
                        let tab_w = gpu.measure_text(&tab.name) + 24.0;
                        positions.push((tx, tab_w));
                        let is_active = i == active_tab;
                        let tab_color = if is_active { bg_base } else { bg_tertiary };

                        // 检测hover
                        let is_hover = self.mouse_x >= tx && self.mouse_x < tx + tab_w
                            && self.mouse_y >= tab_y && self.mouse_y < tab_y + tab_h;

                        // hover时显示背景
                        let final_color = if is_hover && !is_active { bg_secondary } else { tab_color };
                        gpu.draw_rect_simple(&mut encoder, &view, tx, tab_y + 2.0, tab_w, tab_h - 2.0, final_color, sw, sh);

                        if is_active {
                            gpu.draw_rect_simple(&mut encoder, &view, tx, tab_y + 2.0, tab_w, 2.0, primary, sw, sh);
                        }

                        gpu.draw_text_simple(&mut encoder, &view, &tab.name, tx + 12.0, Self::text_y_centered(gpu, tab_y + 2.0, tab_h - 2.0), text_primary, sw, sh);
                        tx += tab_w + 1.0;
                    }
                    self.tab_positions[panel_idx] = positions;

                    // View toggle buttons (right side of tab bar)
                    let view_toggle_x = panel_x + panel_w - 68.0;
                    let view_toggle_y = tab_y + 6.0;
                    // List view icon (active)
                    gpu.draw_rect_simple(&mut encoder, &view, view_toggle_x, view_toggle_y, 20.0, 20.0, bg_base, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, view_toggle_x, view_toggle_y, 20.0, 20.0, border, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "☰", view_toggle_x + 4.0, Self::text_y_centered(gpu, view_toggle_y, 20.0), primary, sw, sh);
                    // Grid view icon
                    gpu.draw_rect_simple(&mut encoder, &view, view_toggle_x + 22.0, view_toggle_y, 20.0, 20.0, bg_base, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, view_toggle_x + 22.0, view_toggle_y, 20.0, 20.0, border, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "⊞", view_toggle_x + 26.0, Self::text_y_centered(gpu, view_toggle_y, 20.0), text_tertiary, sw, sh);
                    // Tree view icon
                    gpu.draw_rect_simple(&mut encoder, &view, view_toggle_x + 44.0, view_toggle_y, 20.0, 20.0, bg_base, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, view_toggle_x + 44.0, view_toggle_y, 20.0, 20.0, border, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "⊟", view_toggle_x + 48.0, Self::text_y_centered(gpu, view_toggle_y, 20.0), text_tertiary, sw, sh);

                    // File list area
                    let list_y = tab_y + tab_h;
                    let list_h = panel_h - (breadcrumb_h + tab_h + panel_status_h);
                    gpu.draw_rect_simple(&mut encoder, &view, panel_x, list_y, panel_w, list_h, bg_base, sw, sh);

                    // Column header
                    let header_y = list_y;
                    gpu.draw_rect_simple(&mut encoder, &view, panel_x, header_y, panel_w, header_h, bg_tertiary, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, panel_x, header_y + header_h - 1.0, panel_w, 1.0, border, sw, sh);

                    let scroll_x = self.panel_scroll_x[panel_idx.min(3)];
                    let col_name = panel_x + 36.0;
                    let col_type = panel_x + panel_w * 0.35;
                    let col_size = panel_x + panel_w * 0.55;
                    let col_date = panel_x + panel_w * 0.7;

                    gpu.draw_text_simple(&mut encoder, &view, "名称", col_name - scroll_x, Self::text_y_centered(gpu, header_y, header_h), text_secondary, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "类型", col_type - scroll_x, Self::text_y_centered(gpu, header_y, header_h), text_secondary, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "大小", col_size - scroll_x, Self::text_y_centered(gpu, header_y, header_h), text_secondary, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "修改时间", col_date - scroll_x, Self::text_y_centered(gpu, header_y, header_h), text_secondary, sw, sh);

                    // 从DualPanelManager获取文件数据
                    let panel_snapshot = if panel_idx == 0 {
                        self.dual_panel.left()
                    } else {
                        self.dual_panel.right()
                    };
                    let files = &panel_snapshot.files;

                    // Calculate visible area using VirtualScrollManager
                    let visible_h = panel_h - breadcrumb_h - tab_h - header_h - panel_status_h;
                    let vs = &mut self.virtual_scroll[panel_idx.min(3)];
                    vs.set_viewport_height(visible_h);
                    vs.sync_from_panel(panel_snapshot.scroll_offset, files.len());

                    let (first_visible, last_visible, _) = vs.visible_range();
                    let scroll_y = vs.scroll_offset();
                    let needs_scroll = files.len() as f32 * row_h > visible_h;

                    // Draw file rows using virtual scrolling (only visible rows)
                    for i in first_visible..last_visible {
                        if i >= files.len() { break; }
                        let file = &files[i];
                        let ry = header_y + header_h + i as f32 * row_h - scroll_y;

                        // 请求Shell图标 (仅文件夹，文件使用Nerd Font回退)
                        if file.is_dir && !gpu.atlas.icon_positions.contains_key(&file.name) {
                            gpu.request_shell_icon(&file.path, IconSize::Small);
                        }

                        // 格式化文件大小
                        let size_str = if file.is_dir {
                            String::new()
                        } else if file.size >= 1048576 {
                            format!("{:.1} MB", file.size as f64 / 1048576.0)
                        } else if file.size >= 1024 {
                            format!("{:.1} KB", file.size as f64 / 1024.0)
                        } else {
                            format!("{} B", file.size)
                        };

                        // 获取文件类型描述
                        let ftype = if file.is_dir {
                            "文件夹"
                        } else {
                            match file.name.rsplit('.').next().unwrap_or("") {
                                "rs" | "js" | "ts" | "py" | "java" | "cpp" | "h" => "源代码",
                                "md" | "txt" | "log" => "文本",
                                "jpg" | "jpeg" | "png" | "gif" | "bmp" => "图片",
                                "mp3" | "mp4" | "avi" | "mkv" => "媒体",
                                "pdf" => "PDF",
                                "zip" | "rar" | "7z" => "压缩包",
                                "exe" | "msi" => "应用程序",
                                "json" | "yaml" | "toml" | "xml" => "配置",
                                _ => "文件",
                            }
                        };

                        let is_selected = if self.dual_panel.is_left_active() {
                            panel_idx == 0 && panel_snapshot.selected_indices.contains(&i)
                        } else {
                            panel_idx == 1 && panel_snapshot.selected_indices.contains(&i)
                        };
                        let is_hovered = !is_selected && self.mouse_y >= ry && self.mouse_y < ry + row_h && self.mouse_x > panel_x && self.mouse_x < panel_x + panel_w;
                        let row_color = if is_selected {
                            primary
                        } else if is_hovered {
                            primary_light
                        } else {
                            bg_base
                        };
                        let text_color = if is_selected { white } else { text_primary };
                        let sub_color = if is_selected { [0.8, 0.9, 1.0, 1.0] } else { text_secondary };

                        // Scissor rect for file list content area (below header, within panel)
                        let scissor_x = panel_x as u32;
                        let scissor_y = (header_y + header_h) as u32;
                        let scissor_w = panel_w as u32;
                        let scissor_h = visible_h as u32;

                        // Get icon for this file (use FolderIconComposer for folders)
                        let file_icon = FileIcon::from_path(&file.name);
                        let icon_color = if file.is_dir {
                            // 文件夹使用合成图标颜色 (金黄色)
                            [1.0, 0.8, 0.0, 1.0]
                        } else {
                            file_icon.icon_color()
                        };

                        gpu.draw_rect_simple_with_scissor(&mut encoder, &view, panel_x, ry, panel_w, row_h, row_color, sw, sh, (scissor_x, scissor_y, scissor_w, scissor_h));
                        gpu.draw_rect_simple_with_scissor(&mut encoder, &view, panel_x, ry + row_h - 1.0, panel_w, 1.0, border, sw, sh, (scissor_x, scissor_y, scissor_w, scissor_h));

                        let panel_right = panel_x + panel_w;
                        let icon_y = ry + (row_h - 24.0) / 2.0;
                        let icon_key = format!("folder:{}", file.name);

                        // 文件夹: 使用FolderIconComposer生成真实像素
                        let icon_x = panel_x + 8.0 - scroll_x;
                        if icon_x + 24.0 > panel_x && icon_x < panel_right {
                        if file.is_dir {
                            if let Some(icon_pixels) = self.folder_icon_composer.compose(
                                crate::ui::folder_icons::FolderType::Normal,
                                24,
                                None,
                            ) {
                                // 上传到纹理图集
                                if !gpu.atlas.icon_positions.contains_key(&icon_key) {
                                    if let Some(atlas_pos) = gpu.atlas.upload_icon(
                                        &icon_key, 24, 24, &icon_pixels, &gpu.queue,
                                    ) {
                                        gpu.draw_texture_with_scissor(&mut encoder, &view, &atlas_pos, icon_x, icon_y, 24.0, 24.0, sw, sh, Some((scissor_x, scissor_y, scissor_w, scissor_h)));
                                    }
                                } else if let Some(atlas_pos) = gpu.atlas.icon_positions.get(&icon_key).cloned() {
                                    gpu.draw_texture_with_scissor(&mut encoder, &view, &atlas_pos, icon_x, icon_y, 24.0, 24.0, sw, sh, Some((scissor_x, scissor_y, scissor_w, scissor_h)));
                                }
                            }
                        } else {
                            // 普通文件: 尝试Shell图标或Nerd Font字符
                            gpu.draw_file_icon_with_scissor(&mut encoder, &view, file_icon, icon_color, &file.path, icon_x, icon_y, 24.0, sw, sh, Some((scissor_x, scissor_y, scissor_w, scissor_h)));
                        }
                        }
                        // Simple clipping: only draw text if visible within panel
                        let name_x = col_name - scroll_x;
                        if name_x + 200.0 > panel_x && name_x < panel_right {
                            gpu.draw_text_simple_with_scissor(&mut encoder, &view, &file.name, name_x, Self::text_y_centered(gpu, ry, row_h), text_color, sw, sh, (scissor_x, scissor_y, scissor_w, scissor_h));
                        }
                        let type_x = col_type - scroll_x;
                        if type_x + 120.0 > panel_x && type_x < panel_right {
                            gpu.draw_text_simple_with_scissor(&mut encoder, &view, ftype, type_x, Self::text_y_centered(gpu, ry, row_h), sub_color, sw, sh, (scissor_x, scissor_y, scissor_w, scissor_h));
                        }
                        if !size_str.is_empty() {
                            let size_x = col_size - scroll_x;
                            if size_x + 80.0 > panel_x && size_x < panel_right {
                                gpu.draw_text_simple_with_scissor(&mut encoder, &view, &size_str, size_x, Self::text_y_centered(gpu, ry, row_h), sub_color, sw, sh, (scissor_x, scissor_y, scissor_w, scissor_h));
                            }
                        }
                        let date_x = col_date - scroll_x;
                        if date_x + 100.0 > panel_x && date_x < panel_right {
                            // 格式化修改时间: YYYY-MM-DD HH:MM
                            let date_str = if file.modified > 0 {
                                let dt = chrono::DateTime::from_timestamp(file.modified as i64, 0)
                                    .unwrap_or_default();
                                dt.format("%Y-%m-%d %H:%M").to_string()
                            } else {
                                String::new()
                            };
                            gpu.draw_text_simple_with_scissor(&mut encoder, &view, &date_str, date_x, Self::text_y_centered(gpu, ry, row_h), sub_color, sw, sh, (scissor_x, scissor_y, scissor_w, scissor_h));
                        }
                    }

                    // Draw vertical scrollbar if needed
                    if needs_scroll {
                        let scrollbar_x = panel_x + panel_w - 12.0;
                        let scrollbar_y = header_y + header_h;
                        let scrollbar_h = visible_h;
                        let content_h = files.len() as f32 * row_h;
                        
                        // Scrollbar track
                        gpu.draw_rect_simple(&mut encoder, &view, scrollbar_x, scrollbar_y, 8.0, scrollbar_h, bg_tertiary, sw, sh);
                        
                        // Scrollbar thumb using VirtualScrollManager
                        let thumb_h = (scrollbar_h * visible_h / content_h).max(20.0);
                        let max_scroll = (content_h - visible_h).max(1.0);
                        let thumb_y = scrollbar_y + (scrollbar_h - thumb_h) * scroll_y / max_scroll;
                        gpu.draw_rect_simple(&mut encoder, &view, scrollbar_x, thumb_y, 8.0, thumb_h, text_tertiary, sw, sh);
                    }

                    // Draw horizontal scrollbar if needed (content wider than panel)
                    // Content spans from col_name (first column) to col_date + 120.0 (last column end)
                    let h_content_w = (col_date + 120.0 - col_name).max(panel_w);
                    let needs_h_scroll = h_content_w > panel_w + 1.0;
                    // Max scroll: rightmost column (col_date+120) aligns with panel right edge
                    // At max scroll: col_date + 120.0 - scroll_x = panel_x + panel_w
                    // So max_scroll = col_date + 120.0 - panel_x - panel_w = panel_w * 0.7 + 120.0 - panel_w = 120.0 - panel_w * 0.3
                    let max_h_scroll = (120.0 - panel_w * 0.3).max(0.0);
                    // Clamp scroll_x to [0, max_h_scroll]
                    self.panel_scroll_x[panel_idx.min(3)] = self.panel_scroll_x[panel_idx.min(3)].clamp(0.0, max_h_scroll);
                    let scroll_x = self.panel_scroll_x[panel_idx.min(3)];
                    if needs_h_scroll {
                        let h_scrollbar_y = panel_y + panel_h - panel_status_h - 12.0;
                        let h_scrollbar_w = panel_w - if needs_scroll { 12.0 } else { 0.0 };
                        
                        // Scrollbar track
                        gpu.draw_rect_simple(&mut encoder, &view, panel_x, h_scrollbar_y, h_scrollbar_w, 8.0, bg_tertiary, sw, sh);
                        
                        // Scrollbar thumb
                        let thumb_w = (h_scrollbar_w * h_scrollbar_w / h_content_w).max(20.0);
                        let thumb_x = panel_x + if max_h_scroll > 0.0 { (h_scrollbar_w - thumb_w) * scroll_x / max_h_scroll } else { 0.0 };
                        gpu.draw_rect_simple(&mut encoder, &view, thumb_x, h_scrollbar_y, thumb_w, 8.0, text_tertiary, sw, sh);
                    }

                    // Panel status bar
                    let panel_status_y = panel_y + panel_h - panel_status_h;
                    gpu.draw_rect_simple(&mut encoder, &view, panel_x, panel_status_y, panel_w, panel_status_h, bg_tertiary, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, panel_x, panel_status_y, panel_w, 1.0, border, sw, sh);
                    let file_count = files.len();
                    let selected_count = panel_snapshot.selected_indices.len();
                    gpu.draw_text_simple(&mut encoder, &view, &format!("{} 个项目", file_count), panel_x + 12.0, Self::text_y_centered(gpu, panel_status_y, panel_status_h), text_tertiary, sw, sh);
                    if selected_count > 0 {
                        gpu.draw_text_simple(&mut encoder, &view, &format!("{} 个选中", selected_count), panel_x + 80.0, Self::text_y_centered(gpu, panel_status_y, panel_status_h), text_tertiary, sw, sh);
                    }
                }
                } else {
                    // === CASCADE MODE ===
                    let cascade_columns = [
                        ("根目录", vec!["C:", "D:", "E:"], 0),
                        ("D:", vec!["work_space", "backup", "tools"], 1),
                        ("work_space", vec!["personal_workspace", "shared"], 2),
                        ("personal_workspace", vec!["zero-explorer", "web-app", "notes.txt"], 3),
                    ];
                    let col_w = 200.0f32;
                    for (col_idx, (header, items, level)) in cascade_columns.iter().enumerate() {
                        let col_x = main_x + col_idx as f32 * (col_w + 1.0);
                        if col_x + col_w > sw { break; }

                        // Column background
                        gpu.draw_rect_simple(&mut encoder, &view, col_x, 0.0, col_w, sh, bg_base, sw, sh);
                        gpu.draw_rect_simple(&mut encoder, &view, col_x + col_w, 0.0, 1.0, sh, border, sw, sh);

                        // Column header
                        gpu.draw_rect_simple(&mut encoder, &view, col_x, 0.0, col_w, 28.0, bg_tertiary, sw, sh);
                        gpu.draw_rect_simple(&mut encoder, &view, col_x, 27.0, col_w, 1.0, border, sw, sh);
                        gpu.draw_text_simple(&mut encoder, &view, header, col_x + 8.0, Self::text_y_centered(gpu, 0.0, 28.0), text_secondary, sw, sh);

                        // Items
                        let mut iy = 28.0f32;
                        for (i, item) in items.iter().enumerate() {
                            if iy + row_h > sh - status_h { break; }
                            let is_selected = self.cascade_selected[*level] == i;
                            let item_bg = if is_selected {
                                primary
                            } else if self.mouse_y >= iy && self.mouse_y < iy + row_h && self.mouse_x >= col_x && self.mouse_x < col_x + col_w {
                                primary_light
                            } else {
                                bg_base
                            };
                            let text_color = if is_selected { white } else { text_primary };
                            
                            // Get icon for this item
                            let file_icon = FileIcon::from_path(item);
                            let icon_color = file_icon.icon_color();
                            
                            gpu.draw_rect_simple(&mut encoder, &view, col_x, iy, col_w, row_h, item_bg, sw, sh);
                            gpu.draw_rect_simple(&mut encoder, &view, col_x, iy + row_h - 1.0, col_w, 1.0, border, sw, sh);
                            let item_icon_y = iy + (row_h - 24.0) / 2.0;
                            gpu.draw_file_icon(&mut encoder, &view, file_icon, icon_color, item, col_x + 8.0, item_icon_y, 24.0, sw, sh);
                            gpu.draw_text_simple(&mut encoder, &view, item, col_x + 42.0, Self::text_y_centered(gpu, iy, row_h), text_color, sw, sh);
                            // Folder arrow
                            if file_icon == FileIcon::Folder {
                                gpu.draw_text_simple(&mut encoder, &view, ">", col_x + col_w - 16.0, Self::text_y_centered(gpu, iy, row_h), text_tertiary, sw, sh);
                            }
                            iy += row_h;
                        }
                    }
                }

                // === MAIN STATUS BAR ===
                let status_y = sh - status_h;
                gpu.draw_rect_simple(&mut encoder, &view, 0.0, status_y, sw, status_h, bg_secondary, sw, sh);
                gpu.draw_rect_simple(&mut encoder, &view, 0.0, status_y, sw, 1.0, border, sw, sh);

                // Mode toggle (left side)
                let mode_x = 12.0;
                let mode_y = status_y + 5.0;
                // Panel mode button
                let panels_active = self.panel_mode == PanelMode::Panels;
                if panels_active {
                    gpu.draw_rect_simple(&mut encoder, &view, mode_x, mode_y, 20.0, 20.0, primary, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "⫿", mode_x + 4.0, Self::text_y_centered(gpu, mode_y, 20.0), white, sw, sh);
                } else {
                    gpu.draw_rect_simple(&mut encoder, &view, mode_x, mode_y, 20.0, 20.0, bg_base, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, mode_x, mode_y, 20.0, 20.0, border, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "⫿", mode_x + 4.0, Self::text_y_centered(gpu, mode_y, 20.0), text_tertiary, sw, sh);
                }
                // Cascade mode button
                let cascade_active = self.panel_mode == PanelMode::Cascade;
                if cascade_active {
                    gpu.draw_rect_simple(&mut encoder, &view, mode_x + 22.0, mode_y, 20.0, 20.0, primary, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "⫴", mode_x + 26.0, Self::text_y_centered(gpu, mode_y, 20.0), white, sw, sh);
                } else {
                    gpu.draw_rect_simple(&mut encoder, &view, mode_x + 22.0, mode_y, 20.0, 20.0, bg_base, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, mode_x + 22.0, mode_y, 20.0, 20.0, border, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "⫴", mode_x + 26.0, Self::text_y_centered(gpu, mode_y, 20.0), text_tertiary, sw, sh);
                }

                // Separator
                gpu.draw_rect_simple(&mut encoder, &view, mode_x + 48.0, status_y + 8.0, 1.0, 14.0, border, sw, sh);

                // Sidebar position toggle
                let sidebar_pos_x = mode_x + 56.0;
                // Left position button (active)
                let left_active = self.sidebar_position == SidebarPosition::Left;
                if left_active {
                    gpu.draw_rect_simple(&mut encoder, &view, sidebar_pos_x, mode_y, 28.0, 20.0, primary, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "◧ 左", sidebar_pos_x + 4.0, Self::text_y_centered(gpu, mode_y, 20.0), white, sw, sh);
                } else {
                    gpu.draw_rect_simple(&mut encoder, &view, sidebar_pos_x, mode_y, 28.0, 20.0, bg_base, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, sidebar_pos_x, mode_y, 28.0, 20.0, border, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "◧ 左", sidebar_pos_x + 4.0, Self::text_y_centered(gpu, mode_y, 20.0), text_tertiary, sw, sh);
                }
                // Right position button
                let right_active = self.sidebar_position == SidebarPosition::Right;
                if right_active {
                    gpu.draw_rect_simple(&mut encoder, &view, sidebar_pos_x + 30.0, mode_y, 28.0, 20.0, primary, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "◨ 右", sidebar_pos_x + 34.0, Self::text_y_centered(gpu, mode_y, 20.0), white, sw, sh);
                } else {
                    gpu.draw_rect_simple(&mut encoder, &view, sidebar_pos_x + 30.0, mode_y, 28.0, 20.0, bg_base, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, sidebar_pos_x + 30.0, mode_y, 28.0, 20.0, border, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "◨ 右", sidebar_pos_x + 34.0, Self::text_y_centered(gpu, mode_y, 20.0), text_tertiary, sw, sh);
                }

                // Separator
                gpu.draw_rect_simple(&mut encoder, &view, sidebar_pos_x + 62.0, status_y + 8.0, 1.0, 14.0, border, sw, sh);

                // Space toggle
                let space_x = sidebar_pos_x + 70.0;
                let spaces = [("default", "[HOME] 默认", primary), ("work", "[WORK] Work", success), ("dev", "[DEV] Dev", warning)];
                let mut sx = space_x;
                for (id, label, color) in spaces.iter() {
                    let is_active = self.active_space == *id;
                    let btn_w = gpu.measure_text(label) + 16.0;
                    if is_active {
                        gpu.draw_rect_simple(&mut encoder, &view, sx, mode_y, btn_w, 20.0, *color, sw, sh);
                        gpu.draw_text_simple(&mut encoder, &view, label, sx + 8.0, Self::text_y_centered(gpu, mode_y, 20.0), white, sw, sh);
                    } else {
                        gpu.draw_rect_simple(&mut encoder, &view, sx, mode_y, btn_w, 20.0, bg_base, sw, sh);
                        gpu.draw_rect_simple(&mut encoder, &view, sx, mode_y, btn_w, 20.0, border, sw, sh);
                        gpu.draw_text_simple(&mut encoder, &view, label, sx + 8.0, Self::text_y_centered(gpu, mode_y, 20.0), text_tertiary, sw, sh);
                    }
                    sx += btn_w + 2.0;
                }
                // Space add button
                gpu.draw_rect_simple(&mut encoder, &view, sx, mode_y, 20.0, 20.0, bg_base, sw, sh);
                gpu.draw_rect_simple(&mut encoder, &view, sx, mode_y, 20.0, 20.0, border, sw, sh);
                gpu.draw_text_simple(&mut encoder, &view, "+", sx + 7.0, Self::text_y_centered(gpu, mode_y, 20.0), text_tertiary, sw, sh);

                // Space dropdown menu (when hovering over space area)
                if self.mouse_x >= space_x && self.mouse_x <= sx + 20.0 && self.mouse_y >= mode_y && self.mouse_y <= mode_y + 20.0 {
                    let dropdown_y = mode_y - 120.0;
                    let dropdown_w = 180.0;
                    gpu.draw_rect_simple(&mut encoder, &view, sx - dropdown_w + 20.0, dropdown_y, dropdown_w, 120.0, bg_base, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, sx - dropdown_w + 20.0, dropdown_y, dropdown_w, 120.0, border, sw, sh);
                    
                    // Dropdown header
                    gpu.draw_rect_simple(&mut encoder, &view, sx - dropdown_w + 20.0, dropdown_y, dropdown_w, 24.0, bg_tertiary, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "空间管理", sx - dropdown_w + 28.0, Self::text_y_centered(gpu, dropdown_y, 24.0), text_secondary, sw, sh);
                    
                    // Dropdown items
                    let spaces = [
                        ("[H]", "默认", primary, true),
                        ("[W]", "Work", success, false),
                        ("[D]", "Dev", warning, false),
                    ];
                    let mut iy = dropdown_y + 28.0;
                    for (icon, name, color, active) in spaces.iter() {
                        let item_bg = if self.mouse_y >= iy && self.mouse_y < iy + 24.0 {
                            primary_light
                        } else if *active {
                            *color
                        } else {
                            bg_base
                        };
                        gpu.draw_rect_simple(&mut encoder, &view, sx - dropdown_w + 20.0, iy, dropdown_w, 24.0, item_bg, sw, sh);
                        gpu.draw_text_simple(&mut encoder, &view, icon, sx - dropdown_w + 28.0, Self::text_y_centered(gpu, iy, 24.0), text_primary, sw, sh);
                        let name_color = if *active { white } else { text_primary };
                        gpu.draw_text_simple(&mut encoder, &view, name, sx - dropdown_w + 48.0, Self::text_y_centered(gpu, iy, 24.0), name_color, sw, sh);
                        iy += 24.0;
                    }
                    
                    // Footer
                    gpu.draw_rect_simple(&mut encoder, &view, sx - dropdown_w + 20.0, iy, dropdown_w, 24.0, bg_tertiary, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "+ 新建空间", sx - dropdown_w + 28.0, Self::text_y_centered(gpu, iy, 24.0), primary, sw, sh);
                }

                // Separator
                gpu.draw_rect_simple(&mut encoder, &view, sx + 24.0, status_y + 8.0, 1.0, 14.0, border, sw, sh);

                // Status info
                let status_info_x = sx + 32.0;
                let panel_count = match self.layout_type {
                    LayoutType::Single => 1,
                    LayoutType::LeftRight => 2,
                    LayoutType::TopBottom => 2,
                    LayoutType::LeftMidRight => 3,
                    LayoutType::Top2Bottom1 => 3,
                    LayoutType::Top1Bottom2 => 3,
                    LayoutType::FourGrid => 4,
                };
                // 显示真实的面板和选中信息
                let status_text = if selected > 0 {
                    format!("{} 个面板 · {} 个选中 · {}", panel_count, selected, active_panel_path)
                } else {
                    format!("{} 个面板 · {} 个项目 · {}", panel_count, total_files, active_panel_path)
                };
                gpu.draw_text_simple(&mut encoder, &view, &status_text, status_info_x, Self::text_y_centered(gpu, status_y, status_h), text_secondary, sw, sh);

                // Layout toggle (right side) - mini grid icons
                let layout_x = sw - 160.0;
                for i in 1..=7u8 {
                    let bx = layout_x + (i - 1) as f32 * 22.0;
                    let is_active = self.layout_type == match i {
                        1 => LayoutType::Single,
                        2 => LayoutType::LeftRight,
                        3 => LayoutType::TopBottom,
                        4 => LayoutType::LeftMidRight,
                        5 => LayoutType::Top2Bottom1,
                        6 => LayoutType::Top1Bottom2,
                        7 => LayoutType::FourGrid,
                        _ => LayoutType::LeftRight,
                    };
                    gpu.draw_rect_simple(&mut encoder, &view, bx, mode_y, 20.0, 20.0, bg_base, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, bx, mode_y, 20.0, 20.0, border, sw, sh);
                    if is_active {
                        gpu.draw_rect_simple(&mut encoder, &view, bx, mode_y, 20.0, 20.0, primary_light, sw, sh);
                        gpu.draw_rect_simple(&mut encoder, &view, bx, mode_y, 20.0, 20.0, primary, sw, sh);
                    }
                    let cell_color = if is_active { white } else { text_tertiary };
                    let (cx, cy, cw, ch) = (bx + 3.0, mode_y + 3.0, 14.0, 14.0);
                    match i {
                        1 => {
                            // 1x1 单面板
                            gpu.draw_rect_simple(&mut encoder, &view, cx, cy, cw, ch, cell_color, sw, sh);
                        }
                        2 => {
                            // 1x2 左右分栏
                            let gap = 1.0;
                            let hw = (cw - gap) / 2.0;
                            gpu.draw_rect_simple(&mut encoder, &view, cx, cy, hw, ch, cell_color, sw, sh);
                            gpu.draw_rect_simple(&mut encoder, &view, cx + hw + gap, cy, hw, ch, cell_color, sw, sh);
                        }
                        3 => {
                            // 2x1 上下分栏
                            let gap = 1.0;
                            let hh = (ch - gap) / 2.0;
                            gpu.draw_rect_simple(&mut encoder, &view, cx, cy, cw, hh, cell_color, sw, sh);
                            gpu.draw_rect_simple(&mut encoder, &view, cx, cy + hh + gap, cw, hh, cell_color, sw, sh);
                        }
                        4 => {
                            // 1x3 左中右分栏
                            let gap = 1.0;
                            let hw = (cw - gap * 2.0) / 3.0;
                            gpu.draw_rect_simple(&mut encoder, &view, cx, cy, hw, ch, cell_color, sw, sh);
                            gpu.draw_rect_simple(&mut encoder, &view, cx + hw + gap, cy, hw, ch, cell_color, sw, sh);
                            gpu.draw_rect_simple(&mut encoder, &view, cx + (hw + gap) * 2.0, cy, hw, ch, cell_color, sw, sh);
                        }
                        5 => {
                            // 上2下1
                            let gap = 1.0;
                            let hw = (cw - gap) / 2.0;
                            let hh = (ch - gap) / 2.0;
                            gpu.draw_rect_simple(&mut encoder, &view, cx, cy, hw, hh, cell_color, sw, sh);
                            gpu.draw_rect_simple(&mut encoder, &view, cx + hw + gap, cy, hw, hh, cell_color, sw, sh);
                            gpu.draw_rect_simple(&mut encoder, &view, cx, cy + hh + gap, cw, hh, cell_color, sw, sh);
                        }
                        6 => {
                            // 上1下2
                            let gap = 1.0;
                            let hw = (cw - gap) / 2.0;
                            let hh = (ch - gap) / 2.0;
                            gpu.draw_rect_simple(&mut encoder, &view, cx, cy, cw, hh, cell_color, sw, sh);
                            gpu.draw_rect_simple(&mut encoder, &view, cx, cy + hh + gap, hw, hh, cell_color, sw, sh);
                            gpu.draw_rect_simple(&mut encoder, &view, cx + hw + gap, cy + hh + gap, hw, hh, cell_color, sw, sh);
                        }
                        7 => {
                            // 2x2 四面板
                            let gap = 1.0;
                            let hw = (cw - gap) / 2.0;
                            let hh = (ch - gap) / 2.0;
                            gpu.draw_rect_simple(&mut encoder, &view, cx, cy, hw, hh, cell_color, sw, sh);
                            gpu.draw_rect_simple(&mut encoder, &view, cx + hw + gap, cy, hw, hh, cell_color, sw, sh);
                            gpu.draw_rect_simple(&mut encoder, &view, cx, cy + hh + gap, hw, hh, cell_color, sw, sh);
                            gpu.draw_rect_simple(&mut encoder, &view, cx + hw + gap, cy + hh + gap, hw, hh, cell_color, sw, sh);
                        }
                        _ => {}
                    }
                }

                // Separator before vim
                gpu.draw_rect_simple(&mut encoder, &view, layout_x - 8.0, status_y + 8.0, 1.0, 14.0, border, sw, sh);

                // === PREVIEW PANEL (overlay on right) ===
                if self.preview_visible {
                    let preview_w = sw * 0.333;
                    let preview_x = sw - preview_w;
                    let preview_h = sh - status_h;

                    // Background
                    gpu.draw_rect_simple(&mut encoder, &view, preview_x, 0.0, preview_w, preview_h, bg_base, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, preview_x, 0.0, 1.0, preview_h, border, sw, sh);

                    // Header
                    gpu.draw_rect_simple(&mut encoder, &view, preview_x, 0.0, preview_w, 36.0, bg_tertiary, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, preview_x, 35.0, preview_w, 1.0, border, sw, sh);

                    // Size toggle buttons (left side of header)
                    let size_btn_y = 8.0;
                    // 1/3 button (active)
                    gpu.draw_rect_simple(&mut encoder, &view, preview_x + 12.0, size_btn_y, 32.0, 20.0, primary, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "1/3", preview_x + 18.0, Self::text_y_centered(gpu, size_btn_y, 20.0), white, sw, sh);
                    // 2/3 button
                    gpu.draw_rect_simple(&mut encoder, &view, preview_x + 46.0, size_btn_y, 32.0, 20.0, bg_base, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, preview_x + 46.0, size_btn_y, 32.0, 20.0, border, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "2/3", preview_x + 52.0, Self::text_y_centered(gpu, size_btn_y, 20.0), text_tertiary, sw, sh);

                    // Close button (right side of header)
                    let close_x = preview_x + preview_w - 28.0;
                    gpu.draw_rect_simple(&mut encoder, &view, close_x, 8.0, 20.0, 20.0, bg_base, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, close_x, 8.0, 20.0, 20.0, border, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "X", close_x + 5.0, Self::text_y_centered(gpu, 8.0, 20.0), text_secondary, sw, sh);

                    // Thumbnail placeholder
                    let thumb_y = 48.0;
                    let thumb_h = 120.0;
                    gpu.draw_rect_simple(&mut encoder, &view, preview_x + 12.0, thumb_y, preview_w - 24.0, thumb_h, bg_tertiary, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, preview_x + 12.0, thumb_y, preview_w - 24.0, thumb_h, border, sw, sh);
                    
                    // Get icon for the preview file
                    let preview_file_icon = FileIcon::from_path("main.rs");
                    let preview_icon_color = preview_file_icon.icon_color();
                    let preview_icon_x = preview_x + preview_w / 2.0 - 20.0;
                    let preview_icon_y = thumb_y + 40.0;
                    gpu.draw_file_icon(&mut encoder, &view, preview_file_icon, preview_icon_color, "main.rs", preview_icon_x, preview_icon_y, 40.0, sw, sh);

                    // File info section
                    let info_y = thumb_y + thumb_h + 16.0;
                    let info_rows = [
                        ("文件名:", "main.rs"),
                        ("类型:", "Rust 源代码文件 (.rs)"),
                        ("大小:", "4.2 KB (4,200 字节)"),
                        ("创建时间:", "2026-08-28 14:20:33"),
                        ("修改时间:", "2026-08-29 09:15:07"),
                        ("访问时间:", "2026-08-31 10:24:15"),
                        ("路径:", "D:\\work_space\\src\\main.rs"),
                    ];
                    let mut iy = info_y;
                    for (label, value) in info_rows.iter() {
                        // Label
                        gpu.draw_text_simple(&mut encoder, &view, label, preview_x + 12.0, Self::text_y_centered(gpu, iy, 22.0), text_tertiary, sw, sh);
                        // Value
                        gpu.draw_text_simple(&mut encoder, &view, value, preview_x + 90.0, Self::text_y_centered(gpu, iy, 22.0), text_primary, sw, sh);
                        iy += 22.0;
                        // Separator line
                        gpu.draw_rect_simple(&mut encoder, &view, preview_x + 12.0, iy - 1.0, preview_w - 24.0, 1.0, border, sw, sh);
                    }

                    // Code preview section
                    let code_y = iy + 12.0;
                    gpu.draw_rect_simple(&mut encoder, &view, preview_x + 12.0, code_y, preview_w - 24.0, 24.0, bg_tertiary, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, preview_x + 12.0, code_y, preview_w - 24.0, 24.0, border, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "内容预览:", preview_x + 20.0, Self::text_y_centered(gpu, code_y, 24.0), text_secondary, sw, sh);

                    // Code content
                    let code_content_y = code_y + 28.0;
                    gpu.draw_rect_simple(&mut encoder, &view, preview_x + 12.0, code_content_y, preview_w - 24.0, 100.0, [0.98, 0.98, 0.98, 1.0], sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, preview_x + 12.0, code_content_y, preview_w - 24.0, 100.0, border, sw, sh);
                    let code_lines = [
                        "use std::env;",
                        "",
                        "mod components;",
                        "mod utils;",
                        "",
                        "fn main() {",
                        "    println!(\"Hello\");",
                        "}",
                    ];
                    let mut ly = code_content_y + 8.0;
                    for line in code_lines.iter() {
                        gpu.draw_text_simple(&mut encoder, &view, line, preview_x + 20.0, ly, [0.3, 0.3, 0.3, 1.0], sw, sh);
                        ly += 14.0;
                    }

                    // Info box at bottom
                    let info_box_y = code_content_y + 112.0;
                    gpu.draw_rect_simple(&mut encoder, &view, preview_x + 12.0, info_box_y, preview_w - 24.0, 48.0, primary_light, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, preview_x + 12.0, info_box_y, preview_w - 24.0, 48.0, primary, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "按 Space 关闭预览", preview_x + 24.0, Self::text_y_centered(gpu, info_box_y + 4.0, 18.0), primary, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "支持 1/3 和 2/3 宽度切换", preview_x + 24.0, Self::text_y_centered(gpu, info_box_y + 24.0, 18.0), text_secondary, sw, sh);
                }

                // === SEARCH PANEL (overlay on center) ===
                if self.search_visible {
                    let search_w = 400.0;
                    let search_h = 300.0;
                    let search_x = (sw - search_w) / 2.0;
                    let search_y = 80.0;

                    // Background overlay (semi-transparent)
                    gpu.draw_rect_simple(&mut encoder, &view, 0.0, 0.0, sw, sh, [0.0, 0.0, 0.0, 0.3], sw, sh);

                    // Search panel background
                    gpu.draw_rect_simple(&mut encoder, &view, search_x, search_y, search_w, search_h, bg_base, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, search_x, search_y, search_w, search_h, border, sw, sh);

                    // Header
                    gpu.draw_rect_simple(&mut encoder, &view, search_x, search_y, search_w, 36.0, bg_tertiary, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, search_x, search_y + 35.0, search_w, 1.0, border, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "快速搜索", search_x + 12.0, Self::text_y_centered(gpu, search_y, 36.0), text_primary, sw, sh);

                    // Close button
                    let close_x = search_x + search_w - 28.0;
                    gpu.draw_rect_simple(&mut encoder, &view, close_x, search_y + 8.0, 20.0, 20.0, bg_base, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, close_x, search_y + 8.0, 20.0, 20.0, border, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "X", close_x + 5.0, Self::text_y_centered(gpu, search_y + 8.0, 20.0), text_secondary, sw, sh);

                    // Search input
                    let input_y = search_y + 48.0;
                    let input_h = 32.0;
                    gpu.draw_rect_simple(&mut encoder, &view, search_x + 12.0, input_y, search_w - 24.0, input_h, bg_base, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, search_x + 12.0, input_y, search_w - 24.0, input_h, border, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "输入文件名...", search_x + 20.0, Self::text_y_centered(gpu, input_y, input_h), text_tertiary, sw, sh);

                    // Search results
                    let results_y = input_y + input_h + 12.0;
                    let results_h = search_h - (results_y - search_y) - 48.0;
                    gpu.draw_rect_simple(&mut encoder, &view, search_x + 12.0, results_y, search_w - 24.0, results_h, bg_base, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, search_x + 12.0, results_y, search_w - 24.0, results_h, border, sw, sh);

                    // Sample search results
                    let search_results = [
                        ("main.rs", "src/main.rs"),
                        ("lib.rs", "src/lib.rs"),
                        ("README.md", "./README.md"),
                        ("logo.png", "assets/logo.png"),
                    ];
                    let mut ry = results_y + 8.0;
                    for (i, (name, path)) in search_results.iter().enumerate() {
                        let item_h = 28.0;
                        if ry + item_h > results_y + results_h { break; }

                        let item_bg = if i == 0 { primary_light } else { bg_base };
                        
                        // Get icon for this search result
                        let file_icon = FileIcon::from_path(name);
                        let icon_color = file_icon.icon_color();
                        
                        gpu.draw_rect_simple(&mut encoder, &view, search_x + 12.0, ry, search_w - 24.0, item_h, item_bg, sw, sh);
                        let search_icon_y = ry + (item_h - 20.0) / 2.0;
                        gpu.draw_file_icon(&mut encoder, &view, file_icon, icon_color, name, search_x + 16.0, search_icon_y, 20.0, sw, sh);
                        gpu.draw_text_simple(&mut encoder, &view, name, search_x + 40.0, Self::text_y_centered(gpu, ry, item_h), text_primary, sw, sh);
                        gpu.draw_text_simple(&mut encoder, &view, path, search_x + 120.0, Self::text_y_centered(gpu, ry, item_h), text_secondary, sw, sh);
                        ry += item_h;
                    }

                    // Footer tips
                    let footer_y = search_y + search_h - 40.0;
                    gpu.draw_rect_simple(&mut encoder, &view, search_x + 12.0, footer_y, search_w - 24.0, 32.0, bg_tertiary, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, search_x + 12.0, footer_y, search_w - 24.0, 32.0, border, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "↑↓ 选择", search_x + 20.0, Self::text_y_centered(gpu, footer_y, 32.0), text_secondary, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "Enter 打开", search_x + 100.0, Self::text_y_centered(gpu, footer_y, 32.0), text_secondary, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "Esc 关闭", search_x + 200.0, Self::text_y_centered(gpu, footer_y, 32.0), text_secondary, sw, sh);
                }

                // === VIM HELP OVERLAY ===
                if self.vim_help_visible {
                    let help_w = 300.0;
                    let help_h = 400.0;
                    let help_x = (sw - help_w) / 2.0;
                    let help_y = 100.0;

                    // Background overlay (semi-transparent)
                    gpu.draw_rect_simple(&mut encoder, &view, 0.0, 0.0, sw, sh, [0.0, 0.0, 0.0, 0.5], sw, sh);

                    // Help panel background (dark theme)
                    gpu.draw_rect_simple(&mut encoder, &view, help_x, help_y, help_w, help_h, [0.12, 0.12, 0.12, 1.0], sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, help_x, help_y, help_w, help_h, [0.3, 0.3, 0.3, 1.0], sw, sh);

                    // Header
                    gpu.draw_rect_simple(&mut encoder, &view, help_x, help_y, help_w, 36.0, [0.15, 0.15, 0.15, 1.0], sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, help_x, help_y + 35.0, help_w, 1.0, [0.3, 0.3, 0.3, 1.0], sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "⌨ Vim 模式帮助", help_x + 12.0, Self::text_y_centered(gpu, help_y, 36.0), [0.9, 0.9, 0.9, 1.0], sw, sh);

                    // Close button
                    let close_x = help_x + help_w - 28.0;
                    gpu.draw_rect_simple(&mut encoder, &view, close_x, help_y + 8.0, 20.0, 20.0, [0.2, 0.2, 0.2, 1.0], sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, close_x, help_y + 8.0, 20.0, 20.0, [0.4, 0.4, 0.4, 1.0], sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "X", close_x + 5.0, Self::text_y_centered(gpu, help_y + 8.0, 20.0), [0.7, 0.7, 0.7, 1.0], sw, sh);

                    // Vim commands
                    let commands = [
                        ("j/k", "上下移动"),
                        ("gg/G", "跳首/跳尾"),
                        ("v", "进入可视选择"),
                        ("", ""), // separator
                        ("y", "复制 (yank)"),
                        ("x", "剪切"),
                        ("p", "粘贴"),
                        ("", ""), // separator
                        ("wv", "竖切分屏"),
                        ("wh", "横切分屏"),
                        ("e", "进入侧栏导航"),
                        ("", ""), // separator
                        ("Esc", "返回 NORMAL"),
                    ];
                    let mut cy = help_y + 48.0;
                    for (key, desc) in commands.iter() {
                        if key.is_empty() {
                            // Separator line
                            gpu.draw_rect_simple(&mut encoder, &view, help_x + 12.0, cy, help_w - 24.0, 1.0, [0.3, 0.3, 0.3, 1.0], sw, sh);
                            cy += 8.0;
                            continue;
                        }

                        let item_h = 28.0;
                        if cy + item_h > help_y + help_h - 48.0 { break; }

                        // Key badge
                        let key_w = gpu.measure_text(key) + 16.0;
                        gpu.draw_rect_simple(&mut encoder, &view, help_x + 12.0, cy + 4.0, key_w, 20.0, [0.2, 0.2, 0.2, 1.0], sw, sh);
                        gpu.draw_rect_simple(&mut encoder, &view, help_x + 12.0, cy + 4.0, key_w, 20.0, [0.4, 0.4, 0.4, 1.0], sw, sh);
                        gpu.draw_text_simple(&mut encoder, &view, key, help_x + 20.0, Self::text_y_centered(gpu, cy + 4.0, 20.0), [0.9, 0.9, 0.9, 1.0], sw, sh);

                        // Description
                        gpu.draw_text_simple(&mut encoder, &view, desc, help_x + 12.0 + key_w + 8.0, Self::text_y_centered(gpu, cy, item_h), [0.7, 0.7, 0.7, 1.0], sw, sh);
                        cy += item_h;
                    }

                    // Footer
                    gpu.draw_rect_simple(&mut encoder, &view, help_x + 12.0, help_y + help_h - 40.0, help_w - 24.0, 32.0, [0.15, 0.15, 0.15, 1.0], sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, help_x + 12.0, help_y + help_h - 40.0, help_w - 24.0, 32.0, [0.3, 0.3, 0.3, 1.0], sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "按 ? 或 Esc 关闭帮助", help_x + 20.0, Self::text_y_centered(gpu, help_y + help_h - 40.0, 32.0), [0.6, 0.6, 0.6, 1.0], sw, sh);
                }

                // Tab close confirmation dialog
                if let Some((panel_idx, tab_idx)) = self.tab_close_confirm {
                    let dialog_w = 320.0;
                    let dialog_h = 140.0;
                    let dialog_x = (sw - dialog_w) / 2.0;
                    let dialog_y = (sh - dialog_h) / 2.0;

                    // 半透明背景
                    gpu.draw_rect_simple(&mut encoder, &view, 0.0, 0.0, sw, sh, [0.0, 0.0, 0.0, 0.5], sw, sh);

                    // 对话框背景
                    gpu.draw_rect_simple(&mut encoder, &view, dialog_x, dialog_y, dialog_w, dialog_h, bg_secondary, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, dialog_x, dialog_y, dialog_w, dialog_h, border, sw, sh);

                    // 标题
                    gpu.draw_text_simple(&mut encoder, &view, "关闭标签页", dialog_x + 16.0, dialog_y + 16.0, text_primary, sw, sh);

                    // 内容
                    let tab_name = if tab_idx < self.panel_tabs[panel_idx].len() {
                        &self.panel_tabs[panel_idx][tab_idx].name
                    } else {
                        "Unknown"
                    };
                    gpu.draw_text_simple(&mut encoder, &view, &format!("确定关闭标签 \"{}\" ?", tab_name), dialog_x + 16.0, dialog_y + 48.0, text_secondary, sw, sh);

                    // 取消按钮
                    let cancel_x = dialog_x + dialog_w - 180.0;
                    let cancel_y = dialog_y + dialog_h - 44.0;
                    let cancel_w = 76.0;
                    let cancel_h = 32.0;
                    let cancel_bg = if self.mouse_x >= cancel_x && self.mouse_x < cancel_x + cancel_w
                        && self.mouse_y >= cancel_y && self.mouse_y < cancel_y + cancel_h {
                        bg_tertiary
                    } else {
                        bg_base
                    };
                    gpu.draw_rect_simple(&mut encoder, &view, cancel_x, cancel_y, cancel_w, cancel_h, cancel_bg, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, cancel_x, cancel_y, cancel_w, cancel_h, border, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "取消", cancel_x + 24.0, Self::text_y_centered(gpu, cancel_y, cancel_h), text_primary, sw, sh);

                    // 确定按钮
                    let confirm_x = dialog_x + dialog_w - 92.0;
                    let confirm_bg = if self.mouse_x >= confirm_x && self.mouse_x < confirm_x + cancel_w
                        && self.mouse_y >= cancel_y && self.mouse_y < cancel_y + cancel_h {
                        [0.8, 0.2, 0.2, 1.0]
                    } else {
                        [0.7, 0.2, 0.2, 1.0]
                    };
                    gpu.draw_rect_simple(&mut encoder, &view, confirm_x, cancel_y, cancel_w, cancel_h, confirm_bg, sw, sh);
                    gpu.draw_rect_simple(&mut encoder, &view, confirm_x, cancel_y, cancel_w, cancel_h, border, sw, sh);
                    gpu.draw_text_simple(&mut encoder, &view, "关闭", confirm_x + 24.0, Self::text_y_centered(gpu, cancel_y, cancel_h), [1.0, 1.0, 1.0, 1.0], sw, sh);
                }

                gpu.queue.submit(std::iter::once(encoder.finish()));
                gpu.end_frame(frame);
            }
        }
    }
}

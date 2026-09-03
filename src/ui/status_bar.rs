use super::components::{Component, ComponentState, Rect};
use crate::core::state::LayoutMode;

#[derive(Debug, Clone, PartialEq)]
pub enum StatusBarLayout {
    Single,
    Dual,
    Triple,
    Quad,
}

impl StatusBarLayout {
    pub fn from_layout_mode(mode: &LayoutMode) -> Self {
        match mode {
            LayoutMode::Single | LayoutMode::Cascade => StatusBarLayout::Single,
            LayoutMode::DualVertical | LayoutMode::DualHorizontal => StatusBarLayout::Dual,
            LayoutMode::TripleLeft
            | LayoutMode::TripleRight
            | LayoutMode::TripleHorizontal
            | LayoutMode::TripleTopTwoBottom
            | LayoutMode::TripleTopOneBottom => StatusBarLayout::Triple,
            LayoutMode::Quad
            | LayoutMode::QuadHorizontal
            | LayoutMode::QuadLeftOneRightThree
            | LayoutMode::QuadTopOneBottomThree => StatusBarLayout::Quad,
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            StatusBarLayout::Single => "单面板",
            StatusBarLayout::Dual => "双面板",
            StatusBarLayout::Triple => "三面板",
            StatusBarLayout::Quad => "四面板",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            StatusBarLayout::Single => "┌─┐",
            StatusBarLayout::Dual => "├─┤",
            StatusBarLayout::Triple => "┌┬┐",
            StatusBarLayout::Quad => "┌┬┐\n└┴┘",
        }
    }
}

#[derive(Debug)]
pub struct StatusBar {
    id: String,
    bounds: Rect,
    state: ComponentState,
    visible: bool,
    panel_count: usize,
    selected_count: usize,
    current_path: String,
    layout: StatusBarLayout,
    /// 布局徽章是否 hover
    layout_badge_hovered: bool,
    /// 布局上下文菜单是否显示
    layout_menu_visible: bool,
    /// 布局模式
    layout_mode: LayoutMode,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            id: "status_bar".to_string(),
            bounds: Rect::default(),
            state: ComponentState::Normal,
            visible: true,
            panel_count: 1,
            selected_count: 0,
            current_path: String::new(),
            layout: StatusBarLayout::Single,
            layout_badge_hovered: false,
            layout_menu_visible: false,
            layout_mode: LayoutMode::Single,
        }
    }

    pub fn set_panel_count(&mut self, count: usize) {
        self.panel_count = count;
    }

    pub fn set_selected_count(&mut self, count: usize) {
        self.selected_count = count;
    }

    pub fn set_current_path(&mut self, path: &str) {
        self.current_path = path.to_string();
    }

    pub fn layout(&self) -> &StatusBarLayout {
        &self.layout
    }

    pub fn set_layout(&mut self, layout: StatusBarLayout) {
        self.layout = layout;
    }

    pub fn set_layout_mode(&mut self, mode: LayoutMode) {
        self.layout = StatusBarLayout::from_layout_mode(&mode);
        self.layout_mode = mode;
    }

    pub fn layout_mode(&self) -> &LayoutMode {
        &self.layout_mode
    }

    pub fn cycle_layout(&mut self) {
        self.layout = match self.layout {
            StatusBarLayout::Single => StatusBarLayout::Dual,
            StatusBarLayout::Dual => StatusBarLayout::Triple,
            StatusBarLayout::Triple => StatusBarLayout::Quad,
            StatusBarLayout::Quad => StatusBarLayout::Single,
        };
    }

    pub fn layout_badge_bounds(&self) -> Rect {
        let badge_width = 80.0;
        let badge_height = 24.0;
        let badge_x = self.bounds.x + self.bounds.width - badge_width - 10.0;
        let badge_y = self.bounds.y + (self.bounds.height - badge_height) / 2.0;

        Rect::new(badge_x, badge_y, badge_width, badge_height)
    }

    pub fn layout_menu_visible(&self) -> bool {
        self.layout_menu_visible
    }

    pub fn show_layout_menu(&mut self) {
        self.layout_menu_visible = true;
    }

    pub fn hide_layout_menu(&mut self) {
        self.layout_menu_visible = false;
    }

    pub fn is_layout_badge_hovered(&self) -> bool {
        self.layout_badge_hovered
    }

    pub fn get_status_text(&self) -> String {
        let panel_info = format!("{} panel(s)", self.panel_count);
        let selected_info = if self.selected_count > 0 {
            format!(", {} selected", self.selected_count)
        } else {
            String::new()
        };
        format!("{}{} | {}", panel_info, selected_info, self.current_path)
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for StatusBar {
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

    fn handle_mouse_button_down(&mut self, x: f32, y: f32) -> bool {
        if self.bounds.contains(x, y) {
            self.set_state(ComponentState::Pressed);

            // 检查布局徽章点击
            let badge_bounds = self.layout_badge_bounds();
            if badge_bounds.contains(x, y) {
                self.show_layout_menu();
                return true;
            }

            true
        } else {
            // 点击外部时关闭布局菜单
            if self.layout_menu_visible {
                self.hide_layout_menu();
                return true;
            }
            false
        }
    }

    fn handle_mouse_button_up(&mut self, _x: f32, _y: f32) -> bool {
        if *self.state() == ComponentState::Pressed {
            self.set_state(ComponentState::Hovered);
            true
        } else {
            false
        }
    }

    fn handle_mouse_move(&mut self, x: f32, y: f32) -> bool {
        if self.bounds.contains(x, y) {
            self.set_state(ComponentState::Hovered);

            // 检查布局徽章 hover
            let badge_bounds = self.layout_badge_bounds();
            self.layout_badge_hovered = badge_bounds.contains(x, y);

            true
        } else {
            if *self.state() == ComponentState::Hovered {
                self.set_state(ComponentState::Normal);
                self.layout_badge_hovered = false;
            }
            false
        }
    }

    fn handle_key_down(&mut self, key: u32) -> bool {
        match key {
            116 => {
                // F5 刷新
                self.cycle_layout();
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_bar_new() {
        let bar = StatusBar::new();
        assert_eq!(bar.id(), "status_bar");
        assert!(bar.visible);
        assert_eq!(bar.layout(), &StatusBarLayout::Single);
        assert_eq!(bar.selected_count, 0);
    }

    #[test]
    fn test_status_bar_layout_cycle() {
        let mut bar = StatusBar::new();
        
        bar.cycle_layout();
        assert_eq!(bar.layout(), &StatusBarLayout::Dual);
        
        bar.cycle_layout();
        assert_eq!(bar.layout(), &StatusBarLayout::Triple);
        
        bar.cycle_layout();
        assert_eq!(bar.layout(), &StatusBarLayout::Quad);
        
        bar.cycle_layout();
        assert_eq!(bar.layout(), &StatusBarLayout::Single);
    }

    #[test]
    fn test_status_bar_layout_from_mode() {
        assert_eq!(
            StatusBarLayout::from_layout_mode(&LayoutMode::Single),
            StatusBarLayout::Single
        );
        assert_eq!(
            StatusBarLayout::from_layout_mode(&LayoutMode::DualVertical),
            StatusBarLayout::Dual
        );
        assert_eq!(
            StatusBarLayout::from_layout_mode(&LayoutMode::TripleHorizontal),
            StatusBarLayout::Triple
        );
        assert_eq!(
            StatusBarLayout::from_layout_mode(&LayoutMode::Quad),
            StatusBarLayout::Quad
        );
    }

    #[test]
    fn test_status_bar_display_name() {
        assert_eq!(StatusBarLayout::Single.display_name(), "单面板");
        assert_eq!(StatusBarLayout::Dual.display_name(), "双面板");
        assert_eq!(StatusBarLayout::Triple.display_name(), "三面板");
        assert_eq!(StatusBarLayout::Quad.display_name(), "四面板");
    }

    #[test]
    fn test_status_bar_badge_bounds() {
        let mut bar = StatusBar::new();
        bar.bounds = Rect::new(0.0, 0.0, 1000.0, 30.0);
        
        let badge = bar.layout_badge_bounds();
        assert!(badge.x > 0.0);
        assert!(badge.width > 0.0);
        assert!(badge.height > 0.0);
    }

    #[test]
    fn test_status_bar_layout_menu() {
        let mut bar = StatusBar::new();
        
        assert!(!bar.layout_menu_visible());
        
        bar.show_layout_menu();
        assert!(bar.layout_menu_visible());
        
        bar.hide_layout_menu();
        assert!(!bar.layout_menu_visible());
    }

    #[test]
    fn test_status_bar_status_text() {
        let mut bar = StatusBar::new();
        bar.set_panel_count(2);
        bar.set_selected_count(5);
        bar.set_current_path("C:\\Users\\test");
        
        let text = bar.get_status_text();
        assert!(text.contains("2 panel(s)"));
        assert!(text.contains("5 selected"));
        assert!(text.contains("C:\\Users\\test"));
    }
}

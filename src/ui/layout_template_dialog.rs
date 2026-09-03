use super::components::{Component, ComponentState, Rect};
use crate::core::state::{LayoutMode, LayoutTemplate};
use std::path::PathBuf;

/// 布局模板对话框 - 参考 Tessoa 的从模板新建布局
#[derive(Debug)]
pub struct LayoutTemplateDialog {
    id: String,
    bounds: Rect,
    state: ComponentState,
    visible: bool,
    /// 所有可用的布局模板
    templates: Vec<LayoutTemplate>,
    /// 当前选中的模板索引
    selected_template: usize,
    /// 各窗格的初始目录
    pane_directories: Vec<PathBuf>,
    /// 当前聚焦的窗格索引
    focused_pane: usize,
    /// 对话框标题
    title: String,
}

impl LayoutTemplateDialog {
    pub fn new() -> Self {
        let templates = LayoutTemplate::all_templates();
        let default_panes = templates[0].panel_count;
        
        Self {
            id: "layout_template_dialog".to_string(),
            bounds: Rect::default(),
            state: ComponentState::Normal,
            visible: false,
            templates,
            selected_template: 0,
            pane_directories: vec![PathBuf::from("C:\\"); default_panes],
            focused_pane: 0,
            title: "从模板新建".to_string(),
        }
    }

    /// 打开对话框
    pub fn open(&mut self) {
        self.visible = true;
        self.selected_template = 0;
        self.update_pane_count();
    }

    /// 关闭对话框
    pub fn close(&mut self) {
        self.visible = false;
    }

    /// 获取选中的模板
    pub fn selected_template(&self) -> &LayoutTemplate {
        &self.templates[self.selected_template]
    }

    /// 获取各窗格的初始目录
    pub fn pane_directories(&self) -> &[PathBuf] {
        &self.pane_directories
    }

    /// 设置窗格目录
    pub fn set_pane_directory(&mut self, pane_index: usize, path: PathBuf) {
        if pane_index < self.pane_directories.len() {
            self.pane_directories[pane_index] = path;
        }
    }

    /// 获取当前聚焦的窗格
    pub fn focused_pane(&self) -> usize {
        self.focused_pane
    }

    /// 设置聚焦的窗格
    pub fn set_focused_pane(&mut self, pane_index: usize) {
        if pane_index < self.pane_directories.len() {
            self.focused_pane = pane_index;
        }
    }

    /// 更新窗格数量以匹配选中的模板
    fn update_pane_count(&mut self) {
        let template = &self.templates[self.selected_template];
        let current_count = self.pane_directories.len();
        let target_count = template.panel_count;

        if target_count > current_count {
            // 添加新窗格，使用主目录
            for _ in current_count..target_count {
                self.pane_directories.push(PathBuf::from("C:\\"));
            }
        } else if target_count < current_count {
            // 移除多余的窗格
            self.pane_directories.truncate(target_count);
        }

        // 确保聚焦的窗格索引有效
        if self.focused_pane >= target_count {
            self.focused_pane = target_count.saturating_sub(1);
        }
    }

    /// 选择模板
    pub fn select_template(&mut self, index: usize) {
        if index < self.templates.len() {
            self.selected_template = index;
            self.update_pane_count();
        }
    }

    /// 获取模板缩略图的边界
    pub fn template_bounds(&self, index: usize) -> Option<Rect> {
        if index >= self.templates.len() {
            return None;
        }

        let item_height = 80.0;
        let item_width = 120.0;
        let padding = 8.0;
        let cols = 3;

        let col = index % cols;
        let row = index / cols;

        let x = self.bounds.x + padding + (col as f32) * (item_width + padding);
        let y = self.bounds.y + 60.0 + padding + (row as f32) * (item_height + padding);

        Some(Rect::new(x, y, item_width, item_height))
    }

    /// 获取窗格预览的边界
    pub fn pane_preview_bounds(&self, pane_index: usize) -> Option<Rect> {
        if pane_index >= self.pane_directories.len() {
            return None;
        }

        let template = &self.templates[self.selected_template];
        let preview_x = self.bounds.x + 420.0;
        let preview_y = self.bounds.y + 60.0;
        let preview_width = 300.0;
        let preview_height = 400.0;

        // 根据模板类型计算各窗格的位置
        let pane_rects = match template.mode {
            LayoutMode::Single => vec![Rect::new(preview_x, preview_y, preview_width, preview_height)],
            LayoutMode::DualVertical => {
                let half_width = (preview_width - 4.0) / 2.0;
                vec![
                    Rect::new(preview_x, preview_y, half_width, preview_height),
                    Rect::new(preview_x + half_width + 4.0, preview_y, half_width, preview_height),
                ]
            }
            LayoutMode::DualHorizontal => {
                let half_height = (preview_height - 4.0) / 2.0;
                vec![
                    Rect::new(preview_x, preview_y, preview_width, half_height),
                    Rect::new(preview_x, preview_y + half_height + 4.0, preview_width, half_height),
                ]
            }
            _ => {
                // 简化处理：均匀分布
                let count = template.panel_count;
                let cols = (count as f32).sqrt().ceil() as usize;
                let rows = (count as f32 / cols as f32).ceil() as usize;
                let cell_width = (preview_width - 4.0 * (cols - 1) as f32) / cols as f32;
                let cell_height = (preview_height - 4.0 * (rows - 1) as f32) / rows as f32;

                let mut rects = Vec::new();
                for i in 0..count {
                    let col = i % cols;
                    let row = i / cols;
                    rects.push(Rect::new(
                        preview_x + col as f32 * (cell_width + 4.0),
                        preview_y + row as f32 * (cell_height + 4.0),
                        cell_width,
                        cell_height,
                    ));
                }
                rects
            }
        };

        pane_rects.get(pane_index).cloned()
    }

    /// 获取创建按钮的边界
    pub fn create_button_bounds(&self) -> Rect {
        Rect::new(
            self.bounds.x + self.bounds.width - 120.0,
            self.bounds.y + self.bounds.height - 50.0,
            100.0,
            36.0,
        )
    }

    /// 获取取消按钮的边界
    pub fn cancel_button_bounds(&self) -> Rect {
        Rect::new(
            self.bounds.x + self.bounds.width - 240.0,
            self.bounds.y + self.bounds.height - 50.0,
            100.0,
            36.0,
        )
    }

    /// 获取浏览按钮的边界
    pub fn browse_button_bounds(&self, pane_index: usize) -> Option<Rect> {
        let pane_bounds = self.pane_preview_bounds(pane_index)?;
        Some(Rect::new(
            pane_bounds.x + pane_bounds.width - 80.0,
            pane_bounds.y + pane_bounds.height - 30.0,
            70.0,
            24.0,
        ))
    }
}

impl Default for LayoutTemplateDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for LayoutTemplateDialog {
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
            self.set_state(ComponentState::Hovered);
            true
        } else {
            if *self.state() == ComponentState::Hovered {
                self.set_state(ComponentState::Normal);
            }
            false
        }
    }

    fn handle_mouse_button_down(&mut self, x: f32, y: f32) -> bool {
        if self.bounds.contains(x, y) {
            self.set_state(ComponentState::Pressed);

            // 检查模板选择
            for (i, _) in self.templates.iter().enumerate() {
                if let Some(bounds) = self.template_bounds(i) {
                    if bounds.contains(x, y) {
                        self.select_template(i);
                        return true;
                    }
                }
            }

            // 检查窗格预览点击
            for i in 0..self.pane_directories.len() {
                if let Some(bounds) = self.pane_preview_bounds(i) {
                    if bounds.contains(x, y) {
                        self.set_focused_pane(i);
                        return true;
                    }
                }
            }

            // 检查创建按钮
            let create_bounds = self.create_button_bounds();
            if create_bounds.contains(x, y) {
                return true;
            }

            // 检查取消按钮
            let cancel_bounds = self.cancel_button_bounds();
            if cancel_bounds.contains(x, y) {
                self.close();
                return true;
            }

            true
        } else {
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

    fn handle_key_down(&mut self, key: u32) -> bool {
        match key {
            27 => {
                // ESC 关闭对话框
                self.close();
                true
            }
            13 => {
                // Enter 创建布局
                // 返回 true 表示创建
                true
            }
            37 => {
                // 左箭头 - 选择上一个模板
                if self.selected_template > 0 {
                    self.select_template(self.selected_template - 1);
                }
                true
            }
            39 => {
                // 右箭头 - 选择下一个模板
                if self.selected_template < self.templates.len() - 1 {
                    self.select_template(self.selected_template + 1);
                }
                true
            }
            9 => {
                // Tab - 在窗格之间轮转
                self.focused_pane = (self.focused_pane + 1) % self.pane_directories.len();
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
    fn test_layout_template_dialog_new() {
        let dialog = LayoutTemplateDialog::new();
        assert_eq!(dialog.id(), "layout_template_dialog");
        assert!(!dialog.visible);
        assert_eq!(dialog.templates.len(), 12);
    }

    #[test]
    fn test_layout_template_dialog_open_close() {
        let mut dialog = LayoutTemplateDialog::new();
        dialog.open();
        assert!(dialog.visible);

        dialog.close();
        assert!(!dialog.visible);
    }

    #[test]
    fn test_layout_template_dialog_select_template() {
        let mut dialog = LayoutTemplateDialog::new();
        dialog.open();

        dialog.select_template(1);
        assert_eq!(dialog.selected_template, 1);
        assert_eq!(dialog.pane_directories.len(), 2); // DualVertical has 2 panes

        dialog.select_template(8);
        assert_eq!(dialog.selected_template, 8);
        assert_eq!(dialog.pane_directories.len(), 4); // Quad has 4 panes
    }

    #[test]
    fn test_layout_template_dialog_pane_directories() {
        let mut dialog = LayoutTemplateDialog::new();
        dialog.open();

        // 设置窗格目录
        dialog.set_pane_directory(0, PathBuf::from("D:\\Projects"));
        assert_eq!(dialog.pane_directories[0], PathBuf::from("D:\\Projects"));

        // 设置无效索引
        dialog.set_pane_directory(10, PathBuf::from("Invalid"));
        assert_eq!(dialog.pane_directories.len(), 1); // 不应该改变
    }

    #[test]
    fn test_layout_template_dialog_focused_pane() {
        let mut dialog = LayoutTemplateDialog::new();
        dialog.open();

        dialog.select_template(1); // DualVertical
        assert_eq!(dialog.focused_pane(), 0);

        dialog.set_focused_pane(1);
        assert_eq!(dialog.focused_pane(), 1);

        // 设置无效索引
        dialog.set_focused_pane(5);
        assert_eq!(dialog.focused_pane(), 1); // 不应该改变
    }

    #[test]
    fn test_layout_template_dialog_bounds() {
        let mut dialog = LayoutTemplateDialog::new();
        dialog.bounds = Rect::new(100.0, 100.0, 800.0, 600.0);

        // 模板边界
        let template_bounds = dialog.template_bounds(0);
        assert!(template_bounds.is_some());

        // 创建按钮边界
        let create_bounds = dialog.create_button_bounds();
        assert!(create_bounds.x > dialog.bounds.x);

        // 取消按钮边界
        let cancel_bounds = dialog.cancel_button_bounds();
        assert!(cancel_bounds.x < create_bounds.x);
    }

    #[test]
    fn test_layout_template_dialog_keyboard() {
        let mut dialog = LayoutTemplateDialog::new();
        dialog.open();

        // ESC 关闭
        assert!(dialog.handle_key_down(27));
        assert!(!dialog.visible);

        // 重新打开
        dialog.open();

        // 右箭头选择下一个模板
        assert!(dialog.handle_key_down(39));
        assert_eq!(dialog.selected_template, 1);

        // 左箭头选择上一个模板
        assert!(dialog.handle_key_down(37));
        assert_eq!(dialog.selected_template, 0);

        // Tab 轮转窗格
        dialog.select_template(1);
        assert!(dialog.handle_key_down(9));
        assert_eq!(dialog.focused_pane(), 1);
    }
}

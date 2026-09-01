use crate::ui::components::{Component, ComponentState, Rect};

const MIN_PANEL_WIDTH: f32 = 200.0;
const DIVIDER_WIDTH: f32 = 4.0;
const MAX_PANELS: usize = 4;

#[derive(Debug, Clone, PartialEq)]
pub enum DividerDragState {
    None,
    Dragging { index: usize, start_x: f32 },
}

#[derive(Debug)]
pub struct PanelContainer {
    id: String,
    bounds: Rect,
    state: ComponentState,
    visible: bool,
    panels: Vec<PanelInfo>,
    divider_positions: Vec<f32>,
    drag_state: DividerDragState,
    gap: f32,
}

#[derive(Debug, Clone)]
pub(crate) struct PanelInfo {
    pub bounds: Rect,
    pub visible: bool,
    pub min_width: f32,
}

impl PanelInfo {
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            visible: true,
            min_width: MIN_PANEL_WIDTH,
        }
    }
}

impl PanelContainer {
    pub fn new(id: &str, bounds: Rect) -> Self {
        let panels = vec![PanelInfo::new(Rect::new(
            bounds.x,
            bounds.y,
            bounds.width,
            bounds.height,
        ))];
        Self {
            id: id.to_string(),
            bounds,
            state: ComponentState::Normal,
            visible: true,
            panels,
            divider_positions: Vec::new(),
            drag_state: DividerDragState::None,
            gap: 1.0,
        }
    }

    pub fn panel_count(&self) -> usize {
        self.panels.len()
    }

    pub fn panels(&self) -> &[PanelInfo] {
        &self.panels
    }

    pub fn panels_mut(&mut self) -> &mut Vec<PanelInfo> {
        &mut self.panels
    }

    pub fn add_panel(&mut self) -> bool {
        if self.panels.len() >= MAX_PANELS {
            return false;
        }

        self.redistribute_panels(self.panels.len() + 1);
        true
    }

    pub fn remove_panel(&mut self) -> bool {
        if self.panels.len() <= 1 {
            return false;
        }

        self.redistribute_panels(self.panels.len() - 1);
        true
    }

    pub fn set_panel_count(&mut self, count: usize) {
        let count = count.clamp(1, MAX_PANELS);
        self.redistribute_panels(count);
    }

    fn redistribute_panels(&mut self, count: usize) {
        if count == 0 {
            return;
        }

        let available_width = self.bounds.width - (count as f32 - 1.0) * self.gap;
        let panel_width = available_width / count as f32;

        self.panels.clear();
        self.divider_positions.clear();

        for i in 0..count {
            let x = self.bounds.x + i as f32 * (panel_width + self.gap);
            self.panels.push(PanelInfo::new(Rect::new(
                x,
                self.bounds.y,
                panel_width,
                self.bounds.height,
            )));

            if i < count - 1 {
                self.divider_positions
                    .push(x + panel_width + self.gap / 2.0);
            }
        }
    }

    pub fn set_bounds(&mut self, bounds: Rect) {
        let old_width = self.bounds.width;
        let old_x = self.bounds.x;
        let new_x = bounds.x;
        let new_width = bounds.width;
        self.bounds = bounds;

        if old_width > 0.0 && (old_width - new_width).abs() > f32::EPSILON {
            let scale = new_width / old_width;
            for panel in &mut self.panels {
                panel.bounds.x = new_x + (panel.bounds.x - old_x) * scale;
                panel.bounds.width *= scale;
            }

            for pos in &mut self.divider_positions {
                *pos = new_x + (*pos - old_x) * scale;
            }
        }

        for panel in &mut self.panels {
            panel.bounds.height = self.bounds.height;
            panel.bounds.y = self.bounds.y;
        }

        self.clamp_panels();
    }

    fn clamp_panels(&mut self) {
        if self.panels.is_empty() {
            return;
        }

        let total_divider_width = (self.panels.len() as f32 - 1.0) * self.gap;
        let available_width = self.bounds.width - total_divider_width;

        for panel in &mut self.panels {
            if panel.bounds.width < panel.min_width {
                panel.bounds.width = panel.min_width;
            }
        }

        let total_panel_width: f32 = self.panels.iter().map(|p| p.bounds.width).sum();
        if total_panel_width > available_width {
            let scale = available_width / total_panel_width;
            for panel in &mut self.panels {
                panel.bounds.width *= scale;
            }
        }

        self.update_divider_positions();
    }

    fn update_divider_positions(&mut self) {
        self.divider_positions.clear();
        let mut x = self.bounds.x;

        for (i, panel) in self.panels.iter().enumerate() {
            panel.bounds.x;
            x += panel.bounds.width;

            if i < self.panels.len() - 1 {
                self.divider_positions.push(x + self.gap / 2.0);
                x += self.gap;
            }
        }
    }

    pub fn divider_hit_test(&self, x: f32, y: f32) -> Option<usize> {
        if !self.bounds.contains(x, y) {
            return None;
        }

        for (i, &div_pos) in self.divider_positions.iter().enumerate() {
            let half_div = DIVIDER_WIDTH / 2.0;
            if (x - div_pos).abs() <= half_div {
                return Some(i);
            }
        }

        None
    }

    pub fn start_divider_drag(&mut self, index: usize, x: f32) {
        self.drag_state = DividerDragState::Dragging {
            index,
            start_x: x,
        };
    }

    pub fn update_divider_drag(&mut self, x: f32) {
        if let DividerDragState::Dragging { index, start_x: _ } = self.drag_state {
            if index >= self.panels.len() - 1 {
                return;
            }

            let left_min = self.panels[index].min_width;
            let right_min = self.panels[index + 1].min_width;
            let delta = x - self.divider_positions[index];
            let new_left_width = (self.panels[index].bounds.width + delta)
                .clamp(left_min, self.bounds.width - right_min);
            let actual_delta = new_left_width - self.panels[index].bounds.width;

            self.panels[index].bounds.width = new_left_width;
            self.panels[index + 1].bounds.width -= actual_delta;

            self.update_divider_positions();
        }
    }

    pub fn end_divider_drag(&mut self) {
        self.drag_state = DividerDragState::None;
    }

    pub fn is_dragging(&self) -> bool {
        self.drag_state != DividerDragState::None
    }

    pub fn panel_at(&self, x: f32, y: f32) -> Option<usize> {
        if !self.bounds.contains(x, y) {
            return None;
        }

        for (i, panel) in self.panels.iter().enumerate() {
            if panel.visible && panel.bounds.contains(x, y) {
                return Some(i);
            }
        }

        None
    }

    pub fn gap(&self) -> f32 {
        self.gap
    }

    pub fn set_gap(&mut self, gap: f32) {
        self.gap = gap;
        self.update_divider_positions();
    }
}

impl Component for PanelContainer {
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
        if self.is_dragging() {
            self.update_divider_drag(x);
            return true;
        }

        if let Some(_index) = self.divider_hit_test(x, y) {
            self.set_state(ComponentState::Hovered);
            return true;
        }

        if self.bounds.contains(x, y) {
            self.set_state(ComponentState::Normal);
            return true;
        }

        self.set_state(ComponentState::Normal);
        false
    }

    fn handle_mouse_button_down(&mut self, x: f32, y: f32) -> bool {
        if let Some(index) = self.divider_hit_test(x, y) {
            self.start_divider_drag(index, x);
            self.set_state(ComponentState::Pressed);
            return true;
        }

        if self.bounds.contains(x, y) {
            return true;
        }

        false
    }

    fn handle_mouse_button_up(&mut self, _x: f32, _y: f32) -> bool {
        if self.is_dragging() {
            self.end_divider_drag();
            self.set_state(ComponentState::Normal);
            return true;
        }

        false
    }
}

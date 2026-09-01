use crate::core::state::LayoutMode;
use crate::ui::components::Rect;

#[derive(Debug, Clone)]
pub struct LayoutConstraint {
    pub min_width: f32,
    pub max_width: f32,
    pub min_height: f32,
    pub max_height: f32,
    pub padding: f32,
    pub gap: f32,
}

impl Default for LayoutConstraint {
    fn default() -> Self {
        Self {
            min_width: 200.0,
            max_width: f32::MAX,
            min_height: 100.0,
            max_height: f32::MAX,
            padding: 0.0,
            gap: 1.0,
        }
    }
}

pub struct LayoutEngine;

impl LayoutEngine {
    pub fn calculate_layout(
        mode: LayoutMode,
        bounds: &Rect,
        constraint: &LayoutConstraint,
        sidebar_visible: bool,
        sidebar_width: f32,
    ) -> Vec<Rect> {
        let gap = constraint.gap;
        let mut available_bounds = bounds.clone();

        if sidebar_visible {
            available_bounds.width -= sidebar_width + gap;
        }

        match mode {
            LayoutMode::Single => {
                vec![available_bounds.clone()]
            }
            LayoutMode::DualVertical => {
                let panel_width = (available_bounds.width - gap) / 2.0;
                vec![
                    Rect::new(
                        available_bounds.x,
                        available_bounds.y,
                        panel_width,
                        available_bounds.height,
                    ),
                    Rect::new(
                        available_bounds.x + panel_width + gap,
                        available_bounds.y,
                        panel_width,
                        available_bounds.height,
                    ),
                ]
            }
            LayoutMode::DualHorizontal => {
                let panel_height = (available_bounds.height - gap) / 2.0;
                vec![
                    Rect::new(
                        available_bounds.x,
                        available_bounds.y,
                        available_bounds.width,
                        panel_height,
                    ),
                    Rect::new(
                        available_bounds.x,
                        available_bounds.y + panel_height + gap,
                        available_bounds.width,
                        panel_height,
                    ),
                ]
            }
            LayoutMode::TripleLeft => {
                let left_width = (available_bounds.width - gap) / 3.0;
                let right_width = available_bounds.width - left_width - gap;
                vec![
                    Rect::new(
                        available_bounds.x,
                        available_bounds.y,
                        left_width,
                        available_bounds.height,
                    ),
                    Rect::new(
                        available_bounds.x + left_width + gap,
                        available_bounds.y,
                        right_width / 2.0,
                        available_bounds.height,
                    ),
                    Rect::new(
                        available_bounds.x + left_width + gap + right_width / 2.0 + gap,
                        available_bounds.y,
                        right_width / 2.0,
                        available_bounds.height,
                    ),
                ]
            }
            LayoutMode::TripleRight => {
                let right_width = (available_bounds.width - gap) / 3.0;
                let left_width = available_bounds.width - right_width - gap;
                vec![
                    Rect::new(
                        available_bounds.x,
                        available_bounds.y,
                        left_width / 2.0,
                        available_bounds.height,
                    ),
                    Rect::new(
                        available_bounds.x + left_width / 2.0 + gap,
                        available_bounds.y,
                        left_width / 2.0,
                        available_bounds.height,
                    ),
                    Rect::new(
                        available_bounds.x + left_width + gap,
                        available_bounds.y,
                        right_width,
                        available_bounds.height,
                    ),
                ]
            }
            LayoutMode::Quad => {
                let panel_width = (available_bounds.width - gap) / 2.0;
                let panel_height = (available_bounds.height - gap) / 2.0;
                vec![
                    Rect::new(
                        available_bounds.x,
                        available_bounds.y,
                        panel_width,
                        panel_height,
                    ),
                    Rect::new(
                        available_bounds.x + panel_width + gap,
                        available_bounds.y,
                        panel_width,
                        panel_height,
                    ),
                    Rect::new(
                        available_bounds.x,
                        available_bounds.y + panel_height + gap,
                        panel_width,
                        panel_height,
                    ),
                    Rect::new(
                        available_bounds.x + panel_width + gap,
                        available_bounds.y + panel_height + gap,
                        panel_width,
                        panel_height,
                    ),
                ]
            }
            LayoutMode::Cascade => {
                let panel_width = available_bounds.width * 0.8;
                let panel_height = available_bounds.height * 0.8;
                let offset = 40.0;
                vec![
                    Rect::new(
                        available_bounds.x,
                        available_bounds.y,
                        panel_width,
                        panel_height,
                    ),
                    Rect::new(
                        available_bounds.x + offset,
                        available_bounds.y + offset,
                        panel_width,
                        panel_height,
                    ),
                ]
            }
        }
    }

    pub fn constrain_rect(rect: &Rect, constraint: &LayoutConstraint) -> Rect {
        let width = rect.width.clamp(constraint.min_width, constraint.max_width);
        let height = rect.height.clamp(constraint.min_height, constraint.max_height);
        Rect::new(rect.x, rect.y, width, height)
    }
}

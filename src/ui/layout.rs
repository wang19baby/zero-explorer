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
            LayoutMode::TripleHorizontal => {
                let panel_width = (available_bounds.width - gap * 2.0) / 3.0;
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
                    Rect::new(
                        available_bounds.x + (panel_width + gap) * 2.0,
                        available_bounds.y,
                        panel_width,
                        available_bounds.height,
                    ),
                ]
            }
            LayoutMode::TripleTopTwoBottom => {
                let bottom_height = (available_bounds.height - gap) / 2.0;
                let top_height = bottom_height;
                let half_width = (available_bounds.width - gap) / 2.0;
                vec![
                    Rect::new(
                        available_bounds.x,
                        available_bounds.y,
                        available_bounds.width,
                        top_height,
                    ),
                    Rect::new(
                        available_bounds.x,
                        available_bounds.y + top_height + gap,
                        half_width,
                        bottom_height,
                    ),
                    Rect::new(
                        available_bounds.x + half_width + gap,
                        available_bounds.y + top_height + gap,
                        half_width,
                        bottom_height,
                    ),
                ]
            }
            LayoutMode::TripleTopOneBottom => {
                let top_height = (available_bounds.height - gap) / 2.0;
                let bottom_height = top_height;
                let half_width = (available_bounds.width - gap) / 2.0;
                vec![
                    Rect::new(
                        available_bounds.x,
                        available_bounds.y,
                        half_width,
                        top_height,
                    ),
                    Rect::new(
                        available_bounds.x + half_width + gap,
                        available_bounds.y,
                        half_width,
                        top_height,
                    ),
                    Rect::new(
                        available_bounds.x,
                        available_bounds.y + top_height + gap,
                        available_bounds.width,
                        bottom_height,
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
            LayoutMode::QuadHorizontal => {
                let panel_width = (available_bounds.width - gap * 3.0) / 4.0;
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
                    Rect::new(
                        available_bounds.x + (panel_width + gap) * 2.0,
                        available_bounds.y,
                        panel_width,
                        available_bounds.height,
                    ),
                    Rect::new(
                        available_bounds.x + (panel_width + gap) * 3.0,
                        available_bounds.y,
                        panel_width,
                        available_bounds.height,
                    ),
                ]
            }
            LayoutMode::QuadLeftOneRightThree => {
                let left_width = (available_bounds.width - gap) / 4.0;
                let right_width = available_bounds.width - left_width - gap;
                let third_height = (available_bounds.height - gap * 2.0) / 3.0;
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
                        right_width,
                        third_height,
                    ),
                    Rect::new(
                        available_bounds.x + left_width + gap,
                        available_bounds.y + third_height + gap,
                        right_width,
                        third_height,
                    ),
                    Rect::new(
                        available_bounds.x + left_width + gap,
                        available_bounds.y + (third_height + gap) * 2.0,
                        right_width,
                        third_height,
                    ),
                ]
            }
            LayoutMode::QuadTopOneBottomThree => {
                let top_height = (available_bounds.height - gap) / 4.0;
                let bottom_height = available_bounds.height - top_height - gap;
                let third_width = (available_bounds.width - gap * 2.0) / 3.0;
                vec![
                    Rect::new(
                        available_bounds.x,
                        available_bounds.y,
                        available_bounds.width,
                        top_height,
                    ),
                    Rect::new(
                        available_bounds.x,
                        available_bounds.y + top_height + gap,
                        third_width,
                        bottom_height,
                    ),
                    Rect::new(
                        available_bounds.x + third_width + gap,
                        available_bounds.y + top_height + gap,
                        third_width,
                        bottom_height,
                    ),
                    Rect::new(
                        available_bounds.x + (third_width + gap) * 2.0,
                        available_bounds.y + top_height + gap,
                        third_width,
                        bottom_height,
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

    pub fn split_vertical(bounds: &Rect, ratio: f32, gap: f32) -> (Rect, Rect) {
        let left_width = bounds.width * ratio;
        let right_width = bounds.width - left_width - gap;

        (
            Rect::new(bounds.x, bounds.y, left_width, bounds.height),
            Rect::new(
                bounds.x + left_width + gap,
                bounds.y,
                right_width,
                bounds.height,
            ),
        )
    }

    pub fn split_horizontal(bounds: &Rect, ratio: f32, gap: f32) -> (Rect, Rect) {
        let top_height = bounds.height * ratio;
        let bottom_height = bounds.height - top_height - gap;

        (
            Rect::new(bounds.x, bounds.y, bounds.width, top_height),
            Rect::new(
                bounds.x,
                bounds.y + top_height + gap,
                bounds.width,
                bottom_height,
            ),
        )
    }

    pub fn center_rect(bounds: &Rect, width: f32, height: f32) -> Rect {
        let x = bounds.x + (bounds.width - width) / 2.0;
        let y = bounds.y + (bounds.height - height) / 2.0;
        Rect::new(x, y, width, height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_constraint() -> LayoutConstraint {
        LayoutConstraint {
            min_width: 200.0,
            max_width: f32::MAX,
            min_height: 100.0,
            max_height: f32::MAX,
            padding: 0.0,
            gap: 1.0,
        }
    }

    #[test]
    fn test_single_layout() {
        let bounds = Rect::new(0.0, 0.0, 800.0, 600.0);
        let constraint = default_constraint();
        
        let panels = LayoutEngine::calculate_layout(
            LayoutMode::Single,
            &bounds,
            &constraint,
            false,
            0.0,
        );
        
        assert_eq!(panels.len(), 1);
        assert_eq!(panels[0].width, 800.0);
        assert_eq!(panels[0].height, 600.0);
    }

    #[test]
    fn test_dual_vertical_layout() {
        let bounds = Rect::new(0.0, 0.0, 800.0, 600.0);
        let constraint = default_constraint();
        
        let panels = LayoutEngine::calculate_layout(
            LayoutMode::DualVertical,
            &bounds,
            &constraint,
            false,
            0.0,
        );
        
        assert_eq!(panels.len(), 2);
        // 两个面板应该等宽
        assert!((panels[0].width - panels[1].width).abs() < 0.01);
        // 高度应该相同
        assert_eq!(panels[0].height, panels[1].height);
    }

    #[test]
    fn test_dual_horizontal_layout() {
        let bounds = Rect::new(0.0, 0.0, 800.0, 600.0);
        let constraint = default_constraint();
        
        let panels = LayoutEngine::calculate_layout(
            LayoutMode::DualHorizontal,
            &bounds,
            &constraint,
            false,
            0.0,
        );
        
        assert_eq!(panels.len(), 2);
        // 两个面板应该等高
        assert!((panels[0].height - panels[1].height).abs() < 0.01);
        // 宽度应该相同
        assert_eq!(panels[0].width, panels[1].width);
    }

    #[test]
    fn test_triple_horizontal_layout() {
        let bounds = Rect::new(0.0, 0.0, 900.0, 600.0);
        let constraint = default_constraint();
        
        let panels = LayoutEngine::calculate_layout(
            LayoutMode::TripleHorizontal,
            &bounds,
            &constraint,
            false,
            0.0,
        );
        
        assert_eq!(panels.len(), 3);
        // 三个面板应该等宽
        assert!((panels[0].width - panels[1].width).abs() < 0.01);
        assert!((panels[1].width - panels[2].width).abs() < 0.01);
    }

    #[test]
    fn test_quad_layout() {
        let bounds = Rect::new(0.0, 0.0, 800.0, 800.0);
        let constraint = default_constraint();
        
        let panels = LayoutEngine::calculate_layout(
            LayoutMode::Quad,
            &bounds,
            &constraint,
            false,
            0.0,
        );
        
        assert_eq!(panels.len(), 4);
        // 四个面板应该等大
        let size = panels[0].width;
        for panel in &panels {
            assert!((panel.width - size).abs() < 0.01);
            assert!((panel.height - size).abs() < 0.01);
        }
    }

    #[test]
    fn test_quad_horizontal_layout() {
        let bounds = Rect::new(0.0, 0.0, 1000.0, 600.0);
        let constraint = default_constraint();
        
        let panels = LayoutEngine::calculate_layout(
            LayoutMode::QuadHorizontal,
            &bounds,
            &constraint,
            false,
            0.0,
        );
        
        assert_eq!(panels.len(), 4);
        // 四个面板应该等宽
        let width = panels[0].width;
        for panel in &panels {
            assert!((panel.width - width).abs() < 0.01);
        }
    }

    #[test]
    fn test_with_sidebar() {
        let bounds = Rect::new(0.0, 0.0, 800.0, 600.0);
        let constraint = default_constraint();
        
        let panels = LayoutEngine::calculate_layout(
            LayoutMode::Single,
            &bounds,
            &constraint,
            true,
            200.0,
        );
        
        assert_eq!(panels.len(), 1);
        // 面板宽度应该减去 sidebar 宽度
        assert!((panels[0].width - 599.0).abs() < 0.01); // 800 - 200 - 1(gap)
    }

    #[test]
    fn test_split_vertical() {
        let bounds = Rect::new(0.0, 0.0, 100.0, 100.0);
        let (left, right) = LayoutEngine::split_vertical(&bounds, 0.5, 10.0);
        
        assert_eq!(left.width, 50.0);
        assert_eq!(right.width, 40.0); // 100 - 50 - 10
        assert_eq!(left.height, 100.0);
        assert_eq!(right.height, 100.0);
    }

    #[test]
    fn test_split_horizontal() {
        let bounds = Rect::new(0.0, 0.0, 100.0, 100.0);
        let (top, bottom) = LayoutEngine::split_horizontal(&bounds, 0.5, 10.0);
        
        assert_eq!(top.height, 50.0);
        assert_eq!(bottom.height, 40.0); // 100 - 50 - 10
        assert_eq!(top.width, 100.0);
        assert_eq!(bottom.width, 100.0);
    }

    #[test]
    fn test_center_rect() {
        let bounds = Rect::new(0.0, 0.0, 100.0, 100.0);
        let centered = LayoutEngine::center_rect(&bounds, 50.0, 30.0);
        
        assert_eq!(centered.x, 25.0);
        assert_eq!(centered.y, 35.0);
        assert_eq!(centered.width, 50.0);
        assert_eq!(centered.height, 30.0);
    }

    #[test]
    fn test_constrain_rect() {
        let rect = Rect::new(0.0, 0.0, 50.0, 50.0);
        let constraint = LayoutConstraint {
            min_width: 100.0,
            max_width: 200.0,
            min_height: 100.0,
            max_height: 200.0,
            padding: 0.0,
            gap: 0.0,
        };
        
        let constrained = LayoutEngine::constrain_rect(&rect, &constraint);
        assert_eq!(constrained.width, 100.0); // 被 min_width 限制
        assert_eq!(constrained.height, 100.0); // 被 min_height 限制
    }
}

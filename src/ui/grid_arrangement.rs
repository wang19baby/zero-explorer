/// 网格排布算法 - 参考 Tessoa 的 4 种图标视图排布

/// 网格排布类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridArrangement {
    /// 规则网格：所有格子等大
    RegularGrid,
    /// 等高行：每行等高，宽度按图片比例
    JustifiedRows,
    /// 瀑布流：每列等宽，高度各随图片
    Masonry,
    /// 马赛克拼贴：大小格混排，每隔几张出双倍大格
    Mosaic,
}

impl GridArrangement {
    /// 获取显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::RegularGrid => "规则网格",
            Self::JustifiedRows => "等高行",
            Self::Masonry => "瀑布流",
            Self::Mosaic => "马赛克拼贴",
        }
    }

    /// 从索引获取
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Self::RegularGrid,
            1 => Self::JustifiedRows,
            2 => Self::Masonry,
            3 => Self::Mosaic,
            _ => Self::RegularGrid,
        }
    }

    /// 转换为索引
    pub fn to_index(&self) -> usize {
        match self {
            Self::RegularGrid => 0,
            Self::JustifiedRows => 1,
            Self::Masonry => 2,
            Self::Mosaic => 3,
        }
    }
}

/// 网格项 - 表示一个网格单元
#[derive(Debug, Clone)]
pub struct GridItem {
    pub index: usize,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    /// 图片的原始宽高比（如果有的话）
    pub aspect_ratio: Option<f32>,
}

/// 网格排布计算器
pub struct GridArrangementCalculator;

impl GridArrangementCalculator {
    /// 计算网格布局
    pub fn calculate(
        arrangement: GridArrangement,
        container_width: f32,
        _container_height: f32,
        item_count: usize,
        item_size: f32,
        gap: f32,
        aspect_ratios: &[Option<f32>],
    ) -> Vec<GridItem> {
        match arrangement {
            GridArrangement::RegularGrid => Self::regular_grid(
                container_width,
                item_count,
                item_size,
                gap,
            ),
            GridArrangement::JustifiedRows => Self::justified_rows(
                container_width,
                item_count,
                item_size,
                gap,
                aspect_ratios,
            ),
            GridArrangement::Masonry => Self::masonry(
                container_width,
                item_count,
                item_size,
                gap,
                aspect_ratios,
            ),
            GridArrangement::Mosaic => Self::mosaic(
                container_width,
                item_count,
                item_size,
                gap,
                aspect_ratios,
            ),
        }
    }

    /// 规则网格：所有格子等大
    fn regular_grid(
        container_width: f32,
        item_count: usize,
        item_size: f32,
        gap: f32,
    ) -> Vec<GridItem> {
        if item_count == 0 {
            return Vec::new();
        }

        let cols = ((container_width + gap) / (item_size + gap)).floor().max(1.0) as usize;
        let actual_item_size = (container_width - gap * (cols - 1) as f32) / cols as f32;

        let mut items = Vec::with_capacity(item_count);
        for i in 0..item_count {
            let col = i % cols;
            let row = i / cols;
            let x = col as f32 * (actual_item_size + gap);
            let y = row as f32 * (actual_item_size + gap);

            items.push(GridItem {
                index: i,
                x,
                y,
                width: actual_item_size,
                height: actual_item_size,
                aspect_ratio: None,
            });
        }

        items
    }

    /// 等高行：每行等高，宽度按图片比例
    fn justified_rows(
        container_width: f32,
        item_count: usize,
        item_size: f32,
        gap: f32,
        aspect_ratios: &[Option<f32>],
    ) -> Vec<GridItem> {
        if item_count == 0 {
            return Vec::new();
        }

        let mut items = Vec::with_capacity(item_count);
        let mut current_row = Vec::new();
        let mut current_row_width = 0.0;
        let mut current_y = 0.0;

        for i in 0..item_count {
            let aspect = aspect_ratios.get(i).and_then(|a| *a).unwrap_or(1.0);
            let item_width = item_size * aspect;

            // 检查是否需要换行
            if !current_row.is_empty() && current_row_width + gap + item_width > container_width {
                // 完成当前行
                Self::layout_row(
                    &current_row,
                    &mut items,
                    current_y,
                    container_width,
                    item_size,
                    gap,
                    aspect_ratios,
                );
                current_y += item_size + gap;
                current_row.clear();
                current_row_width = 0.0;
            }

            current_row.push(i);
            if current_row.is_empty() {
                current_row_width = item_width;
            } else {
                current_row_width += gap + item_width;
            }
        }

        // 处理最后一行
        if !current_row.is_empty() {
            Self::layout_row(
                &current_row,
                &mut items,
                current_y,
                container_width,
                item_size,
                gap,
                aspect_ratios,
            );
        }

        items
    }

    /// 布局一行
    fn layout_row(
        row_indices: &[usize],
        items: &mut Vec<GridItem>,
        y: f32,
        container_width: f32,
        item_size: f32,
        gap: f32,
        aspect_ratios: &[Option<f32>],
    ) {
        if row_indices.is_empty() {
            return;
        }

        // 计算行的总宽度
        let total_width: f32 = row_indices.iter().enumerate().map(|(i, &idx)| {
            let aspect = aspect_ratios.get(idx).and_then(|a| *a).unwrap_or(1.0);
            let w = item_size * aspect;
            if i > 0 { w + gap } else { w }
        }).sum();

        // 缩放以适应容器
        let scale = if total_width > container_width {
            container_width / total_width
        } else {
            1.0
        };

        let scaled_item_size = item_size * scale;
        let mut current_x = 0.0;

        for (_i, &idx) in row_indices.iter().enumerate() {
            let aspect = aspect_ratios.get(idx).and_then(|a| *a).unwrap_or(1.0);
            let item_width = scaled_item_size * aspect;

            items.push(GridItem {
                index: idx,
                x: current_x,
                y,
                width: item_width,
                height: scaled_item_size,
                aspect_ratio: Some(aspect),
            });

            current_x += item_width + gap;
        }
    }

    /// 瀑布流：每列等宽，高度各随图片
    fn masonry(
        container_width: f32,
        item_count: usize,
        item_size: f32,
        gap: f32,
        aspect_ratios: &[Option<f32>],
    ) -> Vec<GridItem> {
        if item_count == 0 {
            return Vec::new();
        }

        let cols = ((container_width + gap) / (item_size + gap)).floor().max(1.0) as usize;
        let actual_col_width = (container_width - gap * (cols - 1) as f32) / cols as f32;

        let mut col_heights = vec![0.0f32; cols];
        let mut items = Vec::with_capacity(item_count);

        for i in 0..item_count {
            // 找到最矮的列
            let shortest_col = col_heights.iter()
                .enumerate()
                .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(idx, _)| idx)
                .unwrap_or(0);

            let aspect: f32 = aspect_ratios.get(i).and_then(|a| *a).unwrap_or(1.0);
            let item_height = actual_col_width / aspect;

            let x = shortest_col as f32 * (actual_col_width + gap);
            let y = col_heights[shortest_col];

            items.push(GridItem {
                index: i,
                x,
                y,
                width: actual_col_width,
                height: item_height,
                aspect_ratio: Some(aspect),
            });

            col_heights[shortest_col] += item_height + gap;
        }

        items
    }

    /// 马赛克拼贴：大小格混排，每隔几张出双倍大格
    fn mosaic(
        container_width: f32,
        item_count: usize,
        item_size: f32,
        gap: f32,
        _aspect_ratios: &[Option<f32>],
    ) -> Vec<GridItem> {
        if item_count == 0 {
            return Vec::new();
        }

        // 马赛克模式：每 5 个项目中，第 1 个是 2x2 大格，其余是 1x1 小格
        let pattern_size = 5;
        let big_item_interval = 1; // 每 pattern_size 个项目中第几个是大格

        let cols = ((container_width + gap) / (item_size + gap)).floor().max(2.0) as usize;
        let actual_col_width = (container_width - gap * (cols - 1) as f32) / cols as f32;

        let mut col_heights = vec![0.0f32; cols];
        let mut items = Vec::with_capacity(item_count);
        let mut pattern_pos = 0;

        for i in 0..item_count {
            let is_big = pattern_pos == big_item_interval;
            let scale = if is_big { 2.0 } else { 1.0 };
            let item_width = actual_col_width * scale;
            let item_height = item_width; // 正方形

            // 找到可以容纳这个项目的列
            let start_col = if is_big {
                // 大格需要连续两列
                col_heights.iter()
                    .enumerate()
                    .take(cols - 1)
                    .filter(|(col, _)| *col < cols - 1)
                    .min_by(|a, b| {
                        let max_a = a.1.max(col_heights[a.0 + 1]);
                        let max_b = b.1.max(col_heights[b.0 + 1]);
                        max_a.partial_cmp(&max_b).unwrap()
                    })
                    .map(|(idx, _)| idx)
                    .unwrap_or(0)
            } else {
                // 小格找最矮的列
                col_heights.iter()
                    .enumerate()
                    .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                    .map(|(idx, _)| idx)
                    .unwrap_or(0)
            };

            let x = start_col as f32 * (actual_col_width + gap);
            let y = col_heights[start_col];

            items.push(GridItem {
                index: i,
                x,
                y,
                width: item_width,
                height: item_height,
                aspect_ratio: Some(1.0),
            });

            // 更新列高度
            let end_col = if is_big { start_col + 2 } else { start_col + 1 };
            for col in start_col..end_col.min(cols) {
                col_heights[col] = y + item_height + gap;
            }

            pattern_pos = (pattern_pos + 1) % pattern_size;
        }

        items
    }

    /// 获取总高度（用于滚动）
    pub fn total_height(items: &[GridItem]) -> f32 {
        items.iter()
            .map(|item| item.y + item.height)
            .fold(0.0f32, f32::max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_arrangement_display_name() {
        assert_eq!(GridArrangement::RegularGrid.display_name(), "规则网格");
        assert_eq!(GridArrangement::JustifiedRows.display_name(), "等高行");
        assert_eq!(GridArrangement::Masonry.display_name(), "瀑布流");
        assert_eq!(GridArrangement::Mosaic.display_name(), "马赛克拼贴");
    }

    #[test]
    fn test_grid_arrangement_from_index() {
        assert_eq!(GridArrangement::from_index(0), GridArrangement::RegularGrid);
        assert_eq!(GridArrangement::from_index(1), GridArrangement::JustifiedRows);
        assert_eq!(GridArrangement::from_index(2), GridArrangement::Masonry);
        assert_eq!(GridArrangement::from_index(3), GridArrangement::Mosaic);
        assert_eq!(GridArrangement::from_index(99), GridArrangement::RegularGrid); // 默认
    }

    #[test]
    fn test_grid_arrangement_to_index() {
        assert_eq!(GridArrangement::RegularGrid.to_index(), 0);
        assert_eq!(GridArrangement::JustifiedRows.to_index(), 1);
        assert_eq!(GridArrangement::Masonry.to_index(), 2);
        assert_eq!(GridArrangement::Mosaic.to_index(), 3);
    }

    #[test]
    fn test_regular_grid() {
        let items = GridArrangementCalculator::calculate(
            GridArrangement::RegularGrid,
            200.0, // container_width
            200.0, // container_height
            4,     // item_count
            50.0,  // item_size
            2.0,   // gap
            &[],
        );

        assert_eq!(items.len(), 4);
        // 检查第一个项目位置
        assert_eq!(items[0].x, 0.0);
        assert_eq!(items[0].y, 0.0);
        // 检查布局是否正确
        let cols = ((200.0f32 + 2.0) / (50.0 + 2.0)).floor() as usize;
        let actual_size = (200.0 - 2.0 * (cols - 1) as f32) / cols as f32;
        assert!((items[0].width - actual_size).abs() < 0.01);
    }

    #[test]
    fn test_justified_rows() {
        let items = GridArrangementCalculator::calculate(
            GridArrangement::JustifiedRows,
            200.0,
            200.0,
            3,
            50.0,
            2.0,
            &[Some(1.0), Some(1.0), Some(1.0)],
        );

        assert_eq!(items.len(), 3);
        // 所有项目应在同一行
        assert!((items[0].y - items[1].y).abs() < 0.01);
        assert!((items[1].y - items[2].y).abs() < 0.01);
    }

    #[test]
    fn test_masonry() {
        let items = GridArrangementCalculator::calculate(
            GridArrangement::Masonry,
            200.0,
            200.0,
            4,
            50.0,
            2.0,
            &[Some(1.0), Some(2.0), Some(1.0), Some(0.5)],
        );

        assert_eq!(items.len(), 4);
        // 瀑布流应有不同高度
        let heights: Vec<f32> = items.iter().map(|i| i.height).collect();
        assert!(heights[0] != heights[1] || heights[2] != heights[3]);
    }

    #[test]
    fn test_mosaic() {
        let items = GridArrangementCalculator::calculate(
            GridArrangement::Mosaic,
            200.0,
            200.0,
            6,
            50.0,
            2.0,
            &[],
        );

        assert_eq!(items.len(), 6);
        // 马赛克应有大小不同的项目
        let widths: Vec<f32> = items.iter().map(|i| i.width).collect();
        assert!(widths.iter().any(|w| *w > 50.0)); // 应有大格
    }

    #[test]
    fn test_total_height() {
        let items = vec![
            GridItem { index: 0, x: 0.0, y: 0.0, width: 50.0, height: 50.0, aspect_ratio: None },
            GridItem { index: 1, x: 0.0, y: 60.0, width: 50.0, height: 40.0, aspect_ratio: None },
        ];

        let height = GridArrangementCalculator::total_height(&items);
        assert_eq!(height, 100.0); // 60 + 40
    }

    #[test]
    fn test_empty_items() {
        let items = GridArrangementCalculator::calculate(
            GridArrangement::RegularGrid,
            200.0,
            200.0,
            0, // item_count
            50.0,
            2.0,
            &[],
        );

        assert_eq!(items.len(), 0);
    }
}

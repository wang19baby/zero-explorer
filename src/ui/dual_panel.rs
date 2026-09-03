use std::mem;

/// 面板快照
/// 参考 MTT File Manager 的零分配swap架构
#[derive(Debug, Default)]
pub struct PanelSnapshot {
    /// 当前路径
    pub path: String,
    /// 文件列表
    pub files: Vec<FileEntry>,
    /// 选中的文件索引
    pub selected_indices: Vec<usize>,
    /// 当前焦点索引
    pub focus_index: usize,
    /// 滚动位置
    pub scroll_offset: f32,
    /// 排序方式
    pub sort_by: SortBy,
    /// 排序方向
    pub sort_descending: bool,
    /// 显示模式
    pub view_mode: ViewMode,
}

/// 文件条目
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: u64,
    pub icon_id: u64,
}

/// 排序方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortBy {
    #[default]
    Name,
    Size,
    Modified,
    Type,
}

/// 显示模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    #[default]
    Details,
    List,
    Grid,
    Tiles,
}

/// 双面板管理器
/// 使用零分配swap避免在面板切换时的内存分配
pub struct DualPanelManager {
    /// 左面板快照
    left: PanelSnapshot,
    /// 右面板快照
    right: PanelSnapshot,
    /// 当前活跃面板 (true = 左, false = 右)
    active_is_left: bool,
}

impl DualPanelManager {
    pub fn new() -> Self {
        Self {
            left: PanelSnapshot::default(),
            right: PanelSnapshot::default(),
            active_is_left: true,
        }
    }

    /// 获取当前活跃面板
    pub fn active(&self) -> &PanelSnapshot {
        if self.active_is_left {
            &self.left
        } else {
            &self.right
        }
    }

    /// 获取当前活跃面板 (可变)
    pub fn active_mut(&mut self) -> &mut PanelSnapshot {
        if self.active_is_left {
            &mut self.left
        } else {
            &mut self.right
        }
    }

    /// 获取非活跃面板
    pub fn inactive(&self) -> &PanelSnapshot {
        if self.active_is_left {
            &self.right
        } else {
            &self.left
        }
    }

    /// 切换活跃面板 (零分配swap)
    pub fn swap_active(&mut self) {
        self.active_is_left = !self.active_is_left;
    }

    /// 零分配swap两个面板的数据
    /// 当用户点击非活跃面板时，交换两个面板的内容
    pub fn swap_panels(&mut self) {
        // 使用std::mem::swap进行零分配交换
        mem::swap(&mut self.left, &mut self.right);
    }

    /// 交换路径 (不交换其他状态)
    pub fn swap_paths(&mut self) {
        mem::swap(&mut self.left.path, &mut self.right.path);
    }

    /// 复制活跃面板的路径到非活跃面板
    pub fn copy_path_to_inactive(&mut self) {
        let active_path = self.active().path.clone();
        if self.active_is_left {
            self.right.path = active_path;
        } else {
            self.left.path = active_path;
        }
    }

    /// 设置左面板
    pub fn set_left(&mut self, snapshot: PanelSnapshot) {
        self.left = snapshot;
    }

    /// 设置右面板
    pub fn set_right(&mut self, snapshot: PanelSnapshot) {
        self.right = snapshot;
    }

    /// 获取左面板
    pub fn left(&self) -> &PanelSnapshot {
        &self.left
    }

    /// 获取左面板 (可变)
    pub fn left_mut(&mut self) -> &mut PanelSnapshot {
        &mut self.left
    }

    /// 获取右面板
    pub fn right(&self) -> &PanelSnapshot {
        &self.right
    }

    /// 获取右面板 (可变)
    pub fn right_mut(&mut self) -> &mut PanelSnapshot {
        &mut self.right
    }

    /// 检查是否左面板活跃
    pub fn is_left_active(&self) -> bool {
        self.active_is_left
    }
}

impl Default for DualPanelManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dual_panel_new() {
        let manager = DualPanelManager::new();
        assert!(manager.is_left_active());
    }

    #[test]
    fn test_swap_active() {
        let mut manager = DualPanelManager::new();
        assert!(manager.is_left_active());
        manager.swap_active();
        assert!(!manager.is_left_active());
    }

    #[test]
    fn test_swap_panels() {
        let mut manager = DualPanelManager::new();
        manager.left.path = "C:\\Users".to_string();
        manager.right.path = "D:\\Games".to_string();

        manager.swap_panels();

        assert_eq!(manager.left.path, "D:\\Games");
        assert_eq!(manager.right.path, "C:\\Users");
    }
}

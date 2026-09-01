use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum DiffLineType {
    Same,
    Added,
    Removed,
    Modified,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub line_number: usize,
    pub content: String,
    pub line_type: DiffLineType,
}

impl DiffLine {
    pub fn new(line_number: usize, content: &str, line_type: DiffLineType) -> Self {
        Self {
            line_number,
            content: content.to_string(),
            line_type,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileDiffStatus {
    Identical,
    Modified,
    OnlyInLeft,
    OnlyInRight,
}

#[derive(Debug, Clone)]
pub struct FileDiff {
    pub left_path: PathBuf,
    pub right_path: PathBuf,
    pub status: FileDiffStatus,
    pub lines: Vec<DiffLine>,
}

impl FileDiff {
    pub fn new(left_path: PathBuf, right_path: PathBuf) -> Self {
        Self {
            left_path,
            right_path,
            status: FileDiffStatus::Identical,
            lines: Vec::new(),
        }
    }

    pub fn compare_files(left_content: &str, right_content: &str) -> Self {
        let left_lines: Vec<&str> = left_content.lines().collect();
        let right_lines: Vec<&str> = right_content.lines().collect();
        let mut diff_lines = Vec::new();
        
        let max_lines = left_lines.len().max(right_lines.len());
        let mut left_idx = 0;
        let mut right_idx = 0;
        let mut line_number = 1;
        
        while left_idx < left_lines.len() || right_idx < right_lines.len() {
            let left_line = left_lines.get(left_idx).copied();
            let right_line = right_lines.get(right_idx).copied();
            
            match (left_line, right_line) {
                (Some(l), Some(r)) if l == r => {
                    diff_lines.push(DiffLine::new(line_number, l, DiffLineType::Same));
                    left_idx += 1;
                    right_idx += 1;
                }
                (Some(l), Some(r)) => {
                    diff_lines.push(DiffLine::new(line_number, l, DiffLineType::Modified));
                    left_idx += 1;
                    right_idx += 1;
                }
                (Some(l), None) => {
                    diff_lines.push(DiffLine::new(line_number, l, DiffLineType::Removed));
                    left_idx += 1;
                }
                (None, Some(r)) => {
                    diff_lines.push(DiffLine::new(line_number, r, DiffLineType::Added));
                    right_idx += 1;
                }
                (None, None) => break,
            }
            line_number += 1;
        }
        
        let status = if diff_lines.iter().all(|l| l.line_type == DiffLineType::Same) {
            FileDiffStatus::Identical
        } else {
            FileDiffStatus::Modified
        };
        
        Self {
            left_path: PathBuf::new(),
            right_path: PathBuf::new(),
            status,
            lines: diff_lines,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DirDiffStatus {
    Identical,
    Different,
}

#[derive(Debug, Clone)]
pub struct DirDiffItem {
    pub name: String,
    pub left_path: Option<PathBuf>,
    pub right_path: Option<PathBuf>,
    pub status: FileDiffStatus,
    pub is_directory: bool,
}

impl DirDiffItem {
    pub fn new(name: &str, is_directory: bool) -> Self {
        Self {
            name: name.to_string(),
            left_path: None,
            right_path: None,
            status: FileDiffStatus::Identical,
            is_directory,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DirDiff {
    pub left_path: PathBuf,
    pub right_path: PathBuf,
    pub status: DirDiffStatus,
    pub items: Vec<DirDiffItem>,
}

impl DirDiff {
    pub fn new(left_path: PathBuf, right_path: PathBuf) -> Self {
        Self {
            left_path,
            right_path,
            status: DirDiffStatus::Identical,
            items: Vec::new(),
        }
    }

    pub fn add_item(&mut self, item: DirDiffItem) {
        if item.status != FileDiffStatus::Identical {
            self.status = DirDiffStatus::Different;
        }
        self.items.push(item);
    }

    pub fn different_items(&self) -> Vec<&DirDiffItem> {
        self.items
            .iter()
            .filter(|i| i.status != FileDiffStatus::Identical)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyncDirection {
    LeftToRight,
    RightToLeft,
    Bidirectional,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyncAction {
    CopyLeftToRight,
    CopyRightToLeft,
    DeleteLeft,
    DeleteRight,
    Skip,
}

#[derive(Debug, Clone)]
pub struct SyncOperation {
    pub item: DirDiffItem,
    pub action: SyncAction,
}

impl SyncOperation {
    pub fn new(item: DirDiffItem, action: SyncAction) -> Self {
        Self { item, action }
    }
}

#[derive(Debug, Clone)]
pub struct SyncPlan {
    pub direction: SyncDirection,
    pub operations: Vec<SyncOperation>,
}

impl SyncPlan {
    pub fn new(direction: SyncDirection) -> Self {
        Self {
            direction,
            operations: Vec::new(),
        }
    }

    pub fn add_operation(&mut self, operation: SyncOperation) {
        self.operations.push(operation);
    }

    pub fn execute(&self) -> Result<Vec<SyncOperation>, String> {
        // In real implementation, this would execute the sync operations
        Ok(self.operations.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_diff_compare_identical() {
        let diff = FileDiff::compare_files("line1\nline2\nline3", "line1\nline2\nline3");
        assert_eq!(diff.status, FileDiffStatus::Identical);
        assert!(diff.lines.iter().all(|l| l.line_type == DiffLineType::Same));
    }

    #[test]
    fn test_file_diff_compare_modified() {
        let diff = FileDiff::compare_files("line1\nline2\nline3", "line1\nmodified\nline3");
        assert_eq!(diff.status, FileDiffStatus::Modified);
        assert_eq!(diff.lines.len(), 3);
        assert_eq!(diff.lines[1].line_type, DiffLineType::Modified);
    }

    #[test]
    fn test_file_diff_compare_added() {
        let diff = FileDiff::compare_files("line1\nline3", "line1\nline2\nline3");
        assert_eq!(diff.status, FileDiffStatus::Modified);
        assert_eq!(diff.lines.len(), 3);
        // Note: Simple diff treats this as Modified (line3 vs line2)
        // A proper diff algorithm would detect Added
        assert_eq!(diff.lines[1].line_type, DiffLineType::Modified);
    }

    #[test]
    fn test_file_diff_compare_removed() {
        let diff = FileDiff::compare_files("line1\nline2\nline3", "line1\nline3");
        assert_eq!(diff.status, FileDiffStatus::Modified);
        assert_eq!(diff.lines.len(), 3);
        // Note: Simple diff treats this as Modified (line2 vs line3)
        // A proper diff algorithm would detect Removed
        assert_eq!(diff.lines[1].line_type, DiffLineType::Modified);
    }

    #[test]
    fn test_dir_diff_new() {
        let diff = DirDiff::new(PathBuf::from("/left"), PathBuf::from("/right"));
        assert_eq!(diff.status, DirDiffStatus::Identical);
        assert!(diff.items.is_empty());
    }

    #[test]
    fn test_dir_diff_add_item() {
        let mut diff = DirDiff::new(PathBuf::from("/left"), PathBuf::from("/right"));
        
        let item = DirDiffItem::new("file.txt", false);
        diff.add_item(item);
        
        assert_eq!(diff.status, DirDiffStatus::Identical);
        assert_eq!(diff.items.len(), 1);
    }

    #[test]
    fn test_dir_diff_add_different_item() {
        let mut diff = DirDiff::new(PathBuf::from("/left"), PathBuf::from("/right"));
        
        let mut item = DirDiffItem::new("file.txt", false);
        item.status = FileDiffStatus::Modified;
        diff.add_item(item);
        
        assert_eq!(diff.status, DirDiffStatus::Different);
    }

    #[test]
    fn test_sync_plan_new() {
        let plan = SyncPlan::new(SyncDirection::LeftToRight);
        assert_eq!(plan.direction, SyncDirection::LeftToRight);
        assert!(plan.operations.is_empty());
    }

    #[test]
    fn test_sync_plan_add_operation() {
        let mut plan = SyncPlan::new(SyncDirection::LeftToRight);
        let item = DirDiffItem::new("file.txt", false);
        let op = SyncOperation::new(item, SyncAction::CopyLeftToRight);
        plan.add_operation(op);
        
        assert_eq!(plan.operations.len(), 1);
    }
}

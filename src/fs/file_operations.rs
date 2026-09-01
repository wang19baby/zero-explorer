use std::path::{Path, PathBuf};
use std::fs;
use std::io;

#[derive(Debug, Clone, PartialEq)]
pub enum ClipboardAction {
    None,
    Copy(Vec<PathBuf>),
    Cut(Vec<PathBuf>),
}

#[derive(Debug, Clone)]
pub struct FileOperationResult {
    pub success: bool,
    pub message: String,
    pub errors: Vec<String>,
}

impl FileOperationResult {
    pub fn ok(message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            errors: Vec::new(),
        }
    }

    pub fn err(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            errors: Vec::new(),
        }
    }

    pub fn with_error(mut self, error: &str) -> Self {
        self.errors.push(error.to_string());
        self.success = false;
        self
    }
}

pub struct FileOperations {
    clipboard: ClipboardAction,
}

impl FileOperations {
    pub fn new() -> Self {
        Self {
            clipboard: ClipboardAction::None,
        }
    }

    pub fn clipboard(&self) -> &ClipboardAction {
        &self.clipboard
    }

    pub fn set_clipboard(&mut self, action: ClipboardAction) {
        self.clipboard = action;
    }

    pub fn clear_clipboard(&mut self) {
        self.clipboard = ClipboardAction::None;
    }

    pub fn copy(&mut self, paths: Vec<PathBuf>) -> FileOperationResult {
        if paths.is_empty() {
            return FileOperationResult::err("No files selected");
        }

        self.clipboard = ClipboardAction::Copy(paths.clone());
        FileOperationResult::ok(&format!("{} item(s) copied to clipboard", paths.len()))
    }

    pub fn cut(&mut self, paths: Vec<PathBuf>) -> FileOperationResult {
        if paths.is_empty() {
            return FileOperationResult::err("No files selected");
        }

        self.clipboard = ClipboardAction::Cut(paths.clone());
        FileOperationResult::ok(&format!("{} item(s) cut to clipboard", paths.len()))
    }

    pub fn paste(&mut self, destination: &Path) -> FileOperationResult {
        let clipboard = self.clipboard.clone();
        match clipboard {
            ClipboardAction::None => {
                return FileOperationResult::err("Nothing to paste");
            }
            ClipboardAction::Copy(ref paths) => {
                let mut errors = Vec::new();
                for path in paths {
                    if let Err(e) = self.copy_item(path, destination) {
                        errors.push(format!("Failed to copy {}: {}", path.display(), e));
                    }
                }

                if errors.is_empty() {
                    FileOperationResult::ok(&format!("{} item(s) pasted", paths.len()))
                } else {
                    FileOperationResult {
                        success: false,
                        message: format!("{} item(s) pasted with errors", paths.len()),
                        errors,
                    }
                }
            }
            ClipboardAction::Cut(ref paths) => {
                let mut errors = Vec::new();
                for path in paths {
                    if let Err(e) = self.move_item(path, destination) {
                        errors.push(format!("Failed to move {}: {}", path.display(), e));
                    }
                }

                self.clipboard = ClipboardAction::None;

                if errors.is_empty() {
                    FileOperationResult::ok(&format!("{} item(s) moved", paths.len()))
                } else {
                    FileOperationResult {
                        success: false,
                        message: format!("{} item(s) moved with errors", paths.len()),
                        errors,
                    }
                }
            }
        }
    }

    pub fn delete(&self, paths: &[PathBuf]) -> FileOperationResult {
        if paths.is_empty() {
            return FileOperationResult::err("No files selected");
        }

        let mut errors = Vec::new();
        for path in paths {
            let result = if path.is_dir() {
                fs::remove_dir_all(path)
            } else {
                fs::remove_file(path)
            };

            if let Err(e) = result {
                errors.push(format!("Failed to delete {}: {}", path.display(), e));
            }
        }

        if errors.is_empty() {
            FileOperationResult::ok(&format!("{} item(s) deleted", paths.len()))
        } else {
            FileOperationResult {
                success: false,
                message: format!("{} item(s) deleted with errors", paths.len()),
                errors,
            }
        }
    }

    pub fn rename(&self, path: &Path, new_name: &str) -> FileOperationResult {
        if new_name.is_empty() {
            return FileOperationResult::err("Name cannot be empty");
        }

        let parent = match path.parent() {
            Some(p) => p,
            None => return FileOperationResult::err("Cannot determine parent directory"),
        };

        let new_path = parent.join(new_name);

        if new_path.exists() {
            return FileOperationResult::err("A file with that name already exists");
        }

        match fs::rename(path, &new_path) {
            Ok(()) => FileOperationResult::ok(&format!("Renamed to {}", new_name)),
            Err(e) => FileOperationResult::err(&format!("Failed to rename: {}", e)),
        }
    }

    pub fn create_dir(&self, path: &Path, name: &str) -> FileOperationResult {
        if name.is_empty() {
            return FileOperationResult::err("Name cannot be empty");
        }

        let new_path = path.join(name);

        if new_path.exists() {
            return FileOperationResult::err("A folder with that name already exists");
        }

        match fs::create_dir(&new_path) {
            Ok(()) => FileOperationResult::ok(&format!("Created folder {}", name)),
            Err(e) => FileOperationResult::err(&format!("Failed to create folder: {}", e)),
        }
    }

    fn copy_item(&self, source: &Path, destination: &Path) -> io::Result<()> {
        let file_name = match source.file_name() {
            Some(name) => name,
            None => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid file name")),
        };

        let dest = destination.join(file_name);

        if source.is_dir() {
            fs::create_dir_all(&dest)?;
            for entry in fs::read_dir(source)? {
                let entry = entry?;
                self.copy_item(&entry.path(), &dest)?;
            }
        } else {
            fs::copy(source, &dest)?;
        }

        Ok(())
    }

    fn move_item(&self, source: &Path, destination: &Path) -> io::Result<()> {
        let file_name = match source.file_name() {
            Some(name) => name,
            None => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid file name")),
        };

        let dest = destination.join(file_name);

        if source.is_dir() {
            fs::create_dir_all(&dest)?;
            for entry in fs::read_dir(source)? {
                let entry = entry?;
                self.move_item(&entry.path(), &dest)?;
            }
            fs::remove_dir(source)?;
        } else {
            fs::rename(source, &dest)?;
        }

        Ok(())
    }
}

impl Default for FileOperations {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_file_operations_new() {
        let ops = FileOperations::new();
        assert_eq!(*ops.clipboard(), ClipboardAction::None);
    }

    #[test]
    fn test_copy_files() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir(&src_dir).unwrap();
        std::fs::write(src_dir.join("test.txt"), "hello").unwrap();
        
        let mut ops = FileOperations::new();
        let result = ops.copy(vec![src_dir.join("test.txt")]);
        
        assert!(result.success);
        assert!(matches!(ops.clipboard(), ClipboardAction::Copy(_)));
    }

    #[test]
    fn test_cut_files() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir(&src_dir).unwrap();
        std::fs::write(src_dir.join("test.txt"), "hello").unwrap();
        
        let mut ops = FileOperations::new();
        let result = ops.cut(vec![src_dir.join("test.txt")]);
        
        assert!(result.success);
        assert!(matches!(ops.clipboard(), ClipboardAction::Cut(_)));
    }

    #[test]
    fn test_paste_copy_files() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        let dst_dir = temp_dir.path().join("dst");
        
        std::fs::create_dir(&src_dir).unwrap();
        std::fs::create_dir(&dst_dir).unwrap();
        std::fs::write(src_dir.join("test.txt"), "hello").unwrap();
        
        let mut ops = FileOperations::new();
        ops.copy(vec![src_dir.join("test.txt")]);
        let result = ops.paste(&dst_dir);
        
        assert!(result.success);
        assert!(dst_dir.join("test.txt").exists());
        assert!(src_dir.join("test.txt").exists()); // Original still exists
    }

    #[test]
    fn test_paste_cut_files() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        let dst_dir = temp_dir.path().join("dst");
        
        std::fs::create_dir(&src_dir).unwrap();
        std::fs::create_dir(&dst_dir).unwrap();
        std::fs::write(src_dir.join("test.txt"), "hello").unwrap();
        
        let mut ops = FileOperations::new();
        ops.cut(vec![src_dir.join("test.txt")]);
        let result = ops.paste(&dst_dir);
        
        assert!(result.success);
        assert!(!src_dir.join("test.txt").exists()); // Original removed
        assert!(dst_dir.join("test.txt").exists());
    }

    #[test]
    fn test_delete_files() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "hello").unwrap();
        
        let ops = FileOperations::new();
        let result = ops.delete(&[file_path.clone()]);
        
        assert!(result.success);
        assert!(!file_path.exists());
    }

    #[test]
    fn test_delete_directory() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("test_dir");
        std::fs::create_dir(&dir_path).unwrap();
        std::fs::write(dir_path.join("file.txt"), "hello").unwrap();
        
        let ops = FileOperations::new();
        let result = ops.delete(&[dir_path.clone()]);
        
        assert!(result.success);
        assert!(!dir_path.exists());
    }

    #[test]
    fn test_rename_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("old_name.txt");
        std::fs::write(&file_path, "hello").unwrap();
        
        let ops = FileOperations::new();
        let result = ops.rename(&file_path, "new_name.txt");
        
        assert!(result.success);
        assert!(!file_path.exists());
        assert!(temp_dir.path().join("new_name.txt").exists());
    }

    #[test]
    fn test_rename_to_existing_name() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        std::fs::write(&file1, "hello").unwrap();
        std::fs::write(&file2, "world").unwrap();
        
        let ops = FileOperations::new();
        let result = ops.rename(&file1, "file2.txt");
        
        assert!(!result.success);
        assert!(result.message.contains("already exists"));
    }

    #[test]
    fn test_create_directory() {
        let temp_dir = TempDir::new().unwrap();
        let new_dir = temp_dir.path().join("new_folder");
        
        let ops = FileOperations::new();
        let result = ops.create_dir(temp_dir.path(), "new_folder");
        
        assert!(result.success);
        assert!(new_dir.exists());
    }

    #[test]
    fn test_create_existing_directory() {
        let temp_dir = TempDir::new().unwrap();
        let new_dir = temp_dir.path().join("existing");
        std::fs::create_dir(&new_dir).unwrap();
        
        let ops = FileOperations::new();
        let result = ops.create_dir(temp_dir.path(), "existing");
        
        assert!(!result.success);
        assert!(result.message.contains("already exists"));
    }

    #[test]
    fn test_paste_empty_clipboard() {
        let temp_dir = TempDir::new().unwrap();
        
        let mut ops = FileOperations::new();
        let result = ops.paste(temp_dir.path());
        
        assert!(!result.success);
        assert!(result.message.contains("Nothing to paste"));
    }

    #[test]
    fn test_copy_empty_list() {
        let mut ops = FileOperations::new();
        let result = ops.copy(vec![]);
        
        assert!(!result.success);
        assert!(result.message.contains("No files selected"));
    }

    #[test]
    fn test_clear_clipboard() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir(&src_dir).unwrap();
        std::fs::write(src_dir.join("test.txt"), "hello").unwrap();
        
        let mut ops = FileOperations::new();
        ops.copy(vec![src_dir.join("test.txt")]);
        assert!(matches!(ops.clipboard(), ClipboardAction::Copy(_)));
        
        ops.clear_clipboard();
        assert_eq!(*ops.clipboard(), ClipboardAction::None);
    }
}

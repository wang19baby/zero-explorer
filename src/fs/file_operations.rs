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

use std::path::{Path, PathBuf};

pub struct PathUtils;

impl PathUtils {
    pub fn normalize(path: &str) -> PathBuf {
        let mut result = PathBuf::new();
        let parts: Vec<&str> = path.split(['/', '\\']).filter(|s| !s.is_empty()).collect();

        for part in parts {
            match part {
                "." => continue,
                ".." => {
                    result.pop();
                }
                p => {
                    // Handle Windows drive letters (e.g., "C:")
                    if p.len() == 2 && p.ends_with(':') {
                        result.push(format!("{}\\", p));
                    } else {
                        result.push(p);
                    }
                }
            }
        }

        result
    }

    pub fn join(base: &Path, relative: &str) -> PathBuf {
        let mut result = base.to_path_buf();
        let parts: Vec<&str> = relative.split(['/', '\\']).filter(|s| !s.is_empty()).collect();

        for part in parts {
            match part {
                "." => continue,
                ".." => {
                    result.pop();
                }
                p => result.push(p),
            }
        }

        result
    }

    pub fn parent(path: &Path) -> Option<PathBuf> {
        path.parent().map(|p| p.to_path_buf())
    }

    pub fn file_name(path: &Path) -> Option<String> {
        path.file_name()
            .map(|n| n.to_string_lossy().to_string())
    }

    pub fn extension(path: &Path) -> Option<String> {
        path.extension()
            .map(|e| e.to_string_lossy().to_string())
    }

    pub fn stem(path: &Path) -> Option<String> {
        path.file_stem()
            .map(|s| s.to_string_lossy().to_string())
    }

    pub fn display_path(path: &Path) -> String {
        path.to_string_lossy().to_string()
    }

    pub fn shorten_path(path: &Path, max_length: usize) -> String {
        let display = Self::display_path(path);
        if display.len() <= max_length {
            return display;
        }

        let parts: Vec<&str> = display.split(['/', '\\']).collect();
        if parts.len() <= 2 {
            return display;
        }

        let mut result = String::new();
        result.push_str(parts[0]);
        result.push('\\');
        result.push_str("...");
        result.push('\\');
        result.push_str(parts.last().unwrap_or(&""));

        if result.len() > max_length {
            let start = &parts[0..2];
            let end = parts.last().unwrap();
            result = format!("{}\\...\\{}", start.join("\\"), end);
        }

        result
    }

    pub fn is_root(path: &Path) -> bool {
        path.parent().is_none()
            || path == Path::new("/")
            || path.to_string_lossy().len() <= 3
    }

    pub fn get_root(path: &Path) -> PathBuf {
        if let Some(parent) = path.parent() {
            if Self::is_root(parent) {
                parent.to_path_buf()
            } else {
                Self::get_root(parent)
            }
        } else {
            path.to_path_buf()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        assert_eq!(
            PathUtils::normalize("C:\\Users\\test"),
            PathBuf::from("C:\\Users\\test")
        );
        assert_eq!(
            PathUtils::normalize("C:\\Users\\..\\test"),
            PathBuf::from("C:\\test")
        );
        assert_eq!(
            PathUtils::normalize("C:\\Users\\.\\test"),
            PathBuf::from("C:\\Users\\test")
        );
    }

    #[test]
    fn test_join() {
        let base = PathBuf::from("C:\\Users");
        assert_eq!(
            PathUtils::join(&base, "test"),
            PathBuf::from("C:\\Users\\test")
        );
        assert_eq!(
            PathUtils::join(&base, "..\\test"),
            PathBuf::from("C:\\test")
        );
    }

    #[test]
    fn test_shorten_path() {
        let path = PathBuf::from("C:\\Users\\John\\Documents\\Projects\\MyProject");
        let shortened = PathUtils::shorten_path(&path, 30);
        assert!(shortened.len() <= 30);
    }
}

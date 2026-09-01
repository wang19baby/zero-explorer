use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    File,
    Directory,
    Symlink,
    Unknown,
}

impl FileType {
    pub fn from_metadata(metadata: &std::fs::Metadata) -> Self {
        if metadata.is_dir() {
            Self::Directory
        } else if metadata.is_file() {
            Self::File
        } else if metadata.is_symlink() {
            Self::Symlink
        } else {
            Self::Unknown
        }
    }

    pub fn is_dir(&self) -> bool {
        *self == Self::Directory
    }

    pub fn is_file(&self) -> bool {
        *self == Self::File
    }
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub path: PathBuf,
    pub file_type: FileType,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub created: Option<SystemTime>,
    pub extension: Option<String>,
    pub is_hidden: bool,
}

impl FileInfo {
    pub fn from_path(path: &Path) -> std::io::Result<Self> {
        let metadata = std::fs::metadata(path)?;
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let extension = path.extension().map(|e| e.to_string_lossy().to_string());

        let is_hidden = name.starts_with('.');

        Ok(Self {
            name,
            path: path.to_path_buf(),
            file_type: FileType::from_metadata(&metadata),
            size: metadata.len(),
            modified: metadata.modified().ok(),
            created: metadata.created().ok(),
            extension,
            is_hidden,
        })
    }

    pub fn display_size(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        match self.size {
            0 => String::from("0 B"),
            s if s < KB => format!("{} B", s),
            s if s < MB => format!("{:.1} KB", s as f64 / KB as f64),
            s if s < GB => format!("{:.1} MB", s as f64 / MB as f64),
            s => format!("{:.2} GB", s as f64 / GB as f64),
        }
    }

    pub fn display_modified(&self) -> String {
        self.modified
            .map(|t| {
                let datetime: chrono::DateTime<chrono::Local> = t.into();
                datetime.format("%Y-%m-%d %H:%M").to_string()
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortBy {
    Name,
    Size,
    Type,
    Modified,
}

pub struct LocalFileSystem;

impl LocalFileSystem {
    pub fn read_dir(path: &Path) -> std::io::Result<Vec<FileInfo>> {
        let mut entries = Vec::new();

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            match FileInfo::from_path(&path) {
                Ok(info) => entries.push(info),
                Err(_) => continue,
            }
        }

        Ok(entries)
    }

    pub fn read_dir_sorted(
        path: &Path,
        sort_by: &SortBy,
        ascending: bool,
    ) -> std::io::Result<Vec<FileInfo>> {
        let mut entries = Self::read_dir(path)?;

        entries.sort_by(|a, b| {
            let cmp = match sort_by {
                SortBy::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortBy::Size => a.size.cmp(&b.size),
                SortBy::Type => match (&a.file_type, &b.file_type) {
                    (FileType::Directory, FileType::Directory) => std::cmp::Ordering::Equal,
                    (FileType::Directory, _) => std::cmp::Ordering::Less,
                    (_, FileType::Directory) => std::cmp::Ordering::Greater,
                    _ => a
                        .extension
                        .as_ref()
                        .unwrap_or(&String::new())
                        .cmp(b.extension.as_ref().unwrap_or(&String::new())),
                },
                SortBy::Modified => {
                    let a_time = a.modified.unwrap_or(SystemTime::UNIX_EPOCH);
                    let b_time = b.modified.unwrap_or(SystemTime::UNIX_EPOCH);
                    a_time.cmp(&b_time)
                }
            };

            if ascending {
                cmp
            } else {
                cmp.reverse()
            }
        });

        entries.sort_by(|a, b| {
            if a.file_type == FileType::Directory && b.file_type != FileType::Directory {
                std::cmp::Ordering::Less
            } else if a.file_type != FileType::Directory && b.file_type == FileType::Directory {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        });

        Ok(entries)
    }

    pub async fn read_dir_async(path: PathBuf) -> std::io::Result<Vec<FileInfo>> {
        tokio::task::spawn_blocking(move || Self::read_dir(&path)).await?
    }

    pub async fn read_dir_sorted_async(
        path: PathBuf,
        sort_by: SortBy,
        ascending: bool,
    ) -> std::io::Result<Vec<FileInfo>> {
        tokio::task::spawn_blocking(move || Self::read_dir_sorted(&path, &sort_by, ascending))
            .await?
    }

    pub fn file_exists(path: &Path) -> bool {
        path.exists()
    }

    pub fn create_dir(path: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(path)?;
        Ok(())
    }

    pub fn remove_file(path: &Path) -> std::io::Result<()> {
        std::fs::remove_file(path)?;
        Ok(())
    }

    pub fn remove_dir(path: &Path) -> std::io::Result<()> {
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    pub fn rename(from: &Path, to: &Path) -> std::io::Result<()> {
        std::fs::rename(from, to)?;
        Ok(())
    }

    pub fn copy(from: &Path, to: &Path) -> std::io::Result<u64> {
        let bytes = std::fs::copy(from, to)?;
        Ok(bytes)
    }

    pub fn get_drives() -> Vec<PathBuf> {
        let mut drives = Vec::new();

        #[cfg(target_os = "windows")]
        {
            for drive in 'A'..='Z' {
                let path = PathBuf::from(format!("{}:\\", drive));
                if path.exists() {
                    drives.push(path);
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            drives.push(PathBuf::from("/"));
        }

        drives
    }

    pub fn get_parent(path: &Path) -> Option<PathBuf> {
        path.parent().map(|p| p.to_path_buf())
    }

    pub fn get_extension(path: &Path) -> Option<String> {
        path.extension().map(|e| e.to_string_lossy().to_string())
    }

    pub fn get_file_name(path: &Path) -> Option<String> {
        path.file_name().map(|n| n.to_string_lossy().to_string())
    }
}

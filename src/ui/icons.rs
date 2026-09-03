use crate::ui::file_list::FileItem;

/// 文件图标类型枚举
/// 使用 Nerd Fonts 图标（TrueType 格式，ab_glyph 可渲染）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileIcon {
    Folder,
    FolderOpen,
    
    // 编程语言
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Html,
    Css,
    Json,
    Yaml,
    Toml,
    Xml,
    C,
    Cpp,
    CSharp,
    Java,
    Go,
    Ruby,
    Php,
    Swift,
    Kotlin,
    
    // 文件类型
    Markdown,
    Text,
    Pdf,
    Word,
    Excel,
    PowerPoint,
    
    // 媒体
    Image,
    Svg,
    Video,
    Audio,
    
    // 其他
    Archive,
    Executable,
    Library,
    
    // 默认
    File,
    Unknown,
}

impl FileIcon {
    /// 根据文件路径推断图标类型
    pub fn from_path(path: &str) -> Self {
        let path_lower = path.to_lowercase();
        
        // 检查是否是目录（通过路径末尾）
        if path.ends_with('/') || path.ends_with('\\') {
            return Self::Folder;
        }
        
        // 获取文件扩展名
        let ext = path_lower.rsplit('.').next().unwrap_or("");
        
        // 如果没有扩展名，可能是目录或无扩展名文件
        // 检查是否是常见的目录名模式
        if ext == path_lower || ext.is_empty() {
            // 检查是否是常见的目录名
            let dir_names = [
                "components", "utils", "src", "lib", "bin", "test", "tests",
                "debug", "release", "build", "dist", "node_modules", "target",
                "docs", "images", "assets", "scripts", "styles", "css", "js",
                "app", "server", "client", "public", "private", "config",
                "2026-08", "2026-07", "2026-06", "2026-05", "2026-04",
                "2026-03", "2026-02", "2026-01", "2025-12", "2025-11",
            ];
            if dir_names.contains(&path_lower.as_str()) {
                return Self::Folder;
            }
            // 如果没有扩展名，返回默认文件图标
            return Self::File;
        }
        
        match ext {
            // 目录
            "dir" | "directory" => Self::Folder,
            
            // 编程语言
            "rs" => Self::Rust,
            "js" | "mjs" | "cjs" => Self::JavaScript,
            "ts" | "mts" | "cts" => Self::TypeScript,
            "py" | "pyw" => Self::Python,
            "html" | "htm" | "xhtml" => Self::Html,
            "css" | "scss" | "sass" | "less" => Self::Css,
            "json" | "jsonc" | "json5" => Self::Json,
            "yaml" | "yml" => Self::Yaml,
            "toml" => Self::Toml,
            "xml" => Self::Xml,
            "c" | "h" => Self::C,
            "cpp" | "cxx" | "cc" | "hpp" | "hxx" => Self::Cpp,
            "cs" => Self::CSharp,
            "java" | "gradle" => Self::Java,
            "go" => Self::Go,
            "rb" | "rake" | "gemspec" => Self::Ruby,
            "php" => Self::Php,
            "swift" => Self::Swift,
            "kt" | "kts" => Self::Kotlin,
            
            // 文档
            "md" | "markdown" | "mdx" => Self::Markdown,
            "txt" | "text" | "log" | "cfg" | "conf" | "ini" => Self::Text,
            "pdf" => Self::Pdf,
            "doc" | "docx" | "odt" | "rtf" => Self::Word,
            "xls" | "xlsx" | "csv" | "ods" => Self::Excel,
            "ppt" | "pptx" | "odp" | "key" => Self::PowerPoint,
            
            // 媒体
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "webp" | "tiff" | "tif" => Self::Image,
            "svg" => Self::Svg,
            "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" | "m4v" => Self::Video,
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" | "opus" => Self::Audio,
            
            // 压缩包
            "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "tgz" | "tar.gz" | "tar.bz2" => Self::Archive,
            
            // 可执行文件
            "exe" | "msi" | "app" | "deb" | "rpm" | "dmg" | "apk" | "ipa" => Self::Executable,
            
            // 库文件
            "dll" | "so" | "dylib" | "lib" | "a" | "o" | "obj" => Self::Library,
            
            // 默认
            _ => Self::File,
        }
    }
    
    /// 获取 Nerd Font 图标字符
    /// 这些图标是 TrueType 格式，ab_glyph 可以渲染
    pub fn icon_char(&self) -> char {
        match self {
            // 文件夹
            Self::Folder => '\u{ea83}',      // nf-cod-folder (works)
            Self::FolderOpen => '\u{ea83}',   // same as folder (eaf7 not in font)
            
            // 编程语言
            Self::Rust => '\u{e7a8}',         // nf-dev-rust (works)
            Self::JavaScript => '\u{e781}',   // nf-dev-javascript (works)
            Self::TypeScript => '\u{e8ca}',   // nf-dev-typescript (works)
            Self::Python => '\u{e73c}',       // nf-dev-python (works)
            Self::Html => '\u{e736}',         // nf-dev-html5 (works)
            Self::Css => '\u{e749}',          // nf-dev-css3 (works)
            Self::Json => '\u{e80b}',         // nf-dev-json (works)
            Self::Yaml => '\u{e8eb}',         // nf-dev-yaml (works)
            Self::Toml => '\u{e7a8}',         // use rust icon as fallback
            Self::Xml => '\u{e736}',          // use html icon as fallback
            Self::C => '\u{e771}',            // nf-dev-c (works)
            Self::Cpp => '\u{e771}',          // use c icon as fallback
            Self::CSharp => '\u{e779}',       // nf-dev-csharp (works)
            Self::Java => '\u{e771}',         // use c icon as fallback
            Self::Go => '\u{e626}',           // nf-custom-go (works)
            Self::Ruby => '\u{e739}',         // nf-dev-ruby (works)
            Self::Php => '\u{e73d}',          // nf-dev-php (works)
            Self::Swift => '\u{e755}',        // nf-dev-swift (works)
            Self::Kotlin => '\u{e73d}',       // use php icon as fallback
            
            // 文档
            Self::Markdown => '\u{e73e}',     // nf-dev-markdown (works)
            Self::Text => '\u{e73e}',         // use markdown icon as fallback
            Self::Pdf => '\u{e73e}',          // use markdown icon as fallback
            Self::Word => '\u{e73e}',         // use markdown icon as fallback
            Self::Excel => '\u{e73e}',        // use markdown icon as fallback
            Self::PowerPoint => '\u{e73e}',   // use markdown icon as fallback
            
            // 媒体
            Self::Image => '\u{e73e}',        // use markdown icon as fallback
            Self::Svg => '\u{e736}',          // use html icon as fallback
            Self::Video => '\u{e7ac}',        // nf-dev-terminal (works)
            Self::Audio => '\u{e7ac}',        // use terminal icon as fallback
            
            // 其他
            Self::Archive => '\u{e7ac}',      // use terminal icon as fallback
            Self::Executable => '\u{e7ac}',   // nf-dev-terminal (works)
            Self::Library => '\u{e7ac}',      // use terminal icon as fallback
            
            // 默认
            Self::File => '\u{e73e}',         // use markdown icon as fallback
            Self::Unknown => '\u{e73e}',      // use markdown icon as fallback
        }
    }
    
    /// 获取图标的颜色（增强饱和度，适合浅色模式）
    pub fn icon_color(&self) -> [f32; 4] {
        match self {
            Self::Folder | Self::FolderOpen => [0.95, 0.80, 0.10, 1.0],     // 金黄色
            
            Self::Rust => [0.80, 0.40, 0.10, 1.0],        // 深橙色
            Self::JavaScript => [0.95, 0.80, 0.05, 1.0],   // 金黄色
            Self::TypeScript => [0.10, 0.55, 0.90, 1.0],   // 深蓝色
            Self::Python => [0.15, 0.50, 0.90, 1.0],       // 深蓝色
            Self::Html => [0.90, 0.30, 0.05, 1.0],         // 深橙色
            Self::Css => [0.15, 0.50, 0.90, 1.0],          // 深蓝色
            Self::Json => [0.95, 0.80, 0.05, 1.0],         // 金黄色
            Self::Yaml => [0.85, 0.15, 0.25, 1.0],         // 深红色
            Self::Toml => [0.15, 0.70, 0.45, 1.0],         // 深绿色
            Self::Xml => [0.10, 0.55, 0.90, 1.0],          // 深蓝色
            Self::C => [0.15, 0.65, 0.95, 1.0],            // 亮蓝色
            Self::Cpp => [0.15, 0.65, 0.95, 1.0],          // 亮蓝色
            Self::CSharp => [0.60, 0.25, 0.85, 1.0],       // 深紫色
            Self::Java => [0.90, 0.30, 0.05, 1.0],         // 深橙色
            Self::Go => [0.05, 0.70, 0.90, 1.0],           // 深青色
            Self::Ruby => [0.90, 0.10, 0.10, 1.0],         // 深红色
            Self::Php => [0.55, 0.25, 0.85, 1.0],          // 深紫色
            Self::Swift => [0.90, 0.30, 0.05, 1.0],        // 深橙色
            Self::Kotlin => [0.60, 0.25, 0.85, 1.0],       // 深紫色
            
            Self::Markdown | Self::Text => [0.45, 0.45, 0.55, 1.0],    // 深灰色
            Self::Pdf => [0.90, 0.15, 0.10, 1.0],           // 深红色
            Self::Word => [0.10, 0.55, 0.90, 1.0],          // 深蓝色
            Self::Excel => [0.10, 0.70, 0.30, 1.0],         // 深绿色
            Self::PowerPoint => [0.90, 0.35, 0.10, 1.0],    // 深橙色
            
            Self::Image | Self::Svg => [0.15, 0.70, 0.45, 1.0],   // 深绿色
            Self::Video => [0.60, 0.25, 0.85, 1.0],         // 深紫色
            Self::Audio => [0.90, 0.40, 0.65, 1.0],         // 深粉色
            
            Self::Archive => [0.15, 0.65, 0.95, 1.0],       // 亮蓝色
            Self::Executable => [0.45, 0.45, 0.55, 1.0],    // 深灰色
            Self::Library => [0.45, 0.45, 0.55, 1.0],       // 深灰色
            
            Self::File | Self::Unknown => [0.45, 0.45, 0.55, 1.0],  // 深灰色
        }
    }
    
    /// 获取图标字符的字符串形式
    pub fn icon_char_str(&self) -> String {
        let ch = self.icon_char();
        let mut buf = [0u8; 4];
        let s = ch.encode_utf8(&mut buf);
        s.to_string()
    }
    
    /// 获取图标的显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Folder => "Folder",
            Self::FolderOpen => "Folder Open",
            Self::Rust => "Rust",
            Self::JavaScript => "JavaScript",
            Self::TypeScript => "TypeScript",
            Self::Python => "Python",
            Self::Html => "HTML",
            Self::Css => "CSS",
            Self::Json => "JSON",
            Self::Yaml => "YAML",
            Self::Toml => "TOML",
            Self::Xml => "XML",
            Self::C => "C",
            Self::Cpp => "C++",
            Self::CSharp => "C#",
            Self::Java => "Java",
            Self::Go => "Go",
            Self::Ruby => "Ruby",
            Self::Php => "PHP",
            Self::Swift => "Swift",
            Self::Kotlin => "Kotlin",
            Self::Markdown => "Markdown",
            Self::Text => "Text",
            Self::Pdf => "PDF",
            Self::Word => "Word",
            Self::Excel => "Excel",
            Self::PowerPoint => "PowerPoint",
            Self::Image => "Image",
            Self::Svg => "SVG",
            Self::Video => "Video",
            Self::Audio => "Audio",
            Self::Archive => "Archive",
            Self::Executable => "Executable",
            Self::Library => "Library",
            Self::File => "File",
            Self::Unknown => "Unknown",
        }
    }
    
    /// 判断是否是目录
    pub fn is_directory(&self) -> bool {
        matches!(self, Self::Folder | Self::FolderOpen)
    }
    
    /// 判断是否是图片文件
    pub fn is_image(&self) -> bool {
        matches!(self, Self::Image | Self::Svg)
    }
    
    /// 判断是否是视频文件
    pub fn is_video(&self) -> bool {
        matches!(self, Self::Video)
    }
    
    /// 判断是否是音频文件
    pub fn is_audio(&self) -> bool {
        matches!(self, Self::Audio)
    }
}

impl From<&FileItem> for FileIcon {
    fn from(item: &FileItem) -> Self {
        if item.is_dir {
            Self::Folder
        } else {
            Self::from_path(&item.name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_file_icon_from_path() {
        // 编程语言
        assert_eq!(FileIcon::from_path("main.rs"), FileIcon::Rust);
        assert_eq!(FileIcon::from_path("app.js"), FileIcon::JavaScript);
        assert_eq!(FileIcon::from_path("index.ts"), FileIcon::TypeScript);
        assert_eq!(FileIcon::from_path("main.py"), FileIcon::Python);
        assert_eq!(FileIcon::from_path("index.html"), FileIcon::Html);
        assert_eq!(FileIcon::from_path("style.css"), FileIcon::Css);
        assert_eq!(FileIcon::from_path("data.json"), FileIcon::Json);
        assert_eq!(FileIcon::from_path("config.yaml"), FileIcon::Yaml);
        assert_eq!(FileIcon::from_path("Cargo.toml"), FileIcon::Toml);
        
        // 文档
        assert_eq!(FileIcon::from_path("README.md"), FileIcon::Markdown);
        assert_eq!(FileIcon::from_path("notes.txt"), FileIcon::Text);
        assert_eq!(FileIcon::from_path("document.pdf"), FileIcon::Pdf);
        
        // 媒体
        assert_eq!(FileIcon::from_path("photo.png"), FileIcon::Image);
        assert_eq!(FileIcon::from_path("video.mp4"), FileIcon::Video);
        assert_eq!(FileIcon::from_path("music.mp3"), FileIcon::Audio);
        
        // 其他
        assert_eq!(FileIcon::from_path("archive.zip"), FileIcon::Archive);
        assert_eq!(FileIcon::from_path("app.exe"), FileIcon::Executable);
        
        // 默认
        assert_eq!(FileIcon::from_path("unknown"), FileIcon::File);
    }
    
    #[test]
    fn test_file_icon_is_directory() {
        assert!(FileIcon::Folder.is_directory());
        assert!(FileIcon::FolderOpen.is_directory());
        assert!(!FileIcon::File.is_directory());
        assert!(!FileIcon::Rust.is_directory());
    }
    
    #[test]
    fn test_file_icon_is_media() {
        assert!(FileIcon::Image.is_image());
        assert!(FileIcon::Svg.is_image());
        assert!(FileIcon::Video.is_video());
        assert!(FileIcon::Audio.is_audio());
        assert!(!FileIcon::File.is_image());
        assert!(!FileIcon::File.is_video());
        assert!(!FileIcon::File.is_audio());
    }
}

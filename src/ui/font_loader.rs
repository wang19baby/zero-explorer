use std::sync::{Arc, Mutex};
use crossbeam_channel::{bounded, Receiver, Sender};

/// 字体加载状态
#[derive(Debug, Clone, PartialEq)]
pub enum FontLoadState {
    /// 未开始
    NotStarted,
    /// 加载中
    Loading,
    /// 加载完成
    Loaded(FontData),
    /// 加载失败
    Failed(String),
}

/// 字体数据
#[derive(Debug, Clone, PartialEq)]
pub struct FontData {
    /// 主字体 (Segoe UI 或系统字体)
    pub primary: Vec<u8>,
    /// CJK回退字体 (简体中文)
    pub cjk_sc: Option<Vec<u8>>,
    /// CJK回退字体 (繁体中文)
    pub cjk_tc: Option<Vec<u8>>,
    /// CJK回退字体 (日文)
    pub cjk_jp: Option<Vec<u8>>,
    /// CJK回退字体 (韩文)
    pub cjk_kr: Option<Vec<u8>>,
    /// 图标字体 (Nerd Font)
    pub icon: Option<Vec<u8>>,
}

/// 异步字体加载器
/// 参考 MTT File Manager 的异步字体加载架构
pub struct AsyncFontLoader {
    state: Arc<Mutex<FontLoadState>>,
    sender: Sender<FontLoadState>,
    receiver: Receiver<FontLoadState>,
}

impl AsyncFontLoader {
    pub fn new() -> Self {
        let (sender, receiver) = bounded(1);
        let state = Arc::new(Mutex::new(FontLoadState::NotStarted));

        Self {
            state,
            sender,
            receiver,
        }
    }

    /// 开始异步加载字体
    pub fn start_loading(&self) {
        let mut state = self.state.lock().unwrap();
        if *state != FontLoadState::NotStarted {
            return;
        }
        *state = FontLoadState::Loading;

        let state_clone = Arc::clone(&self.state);
        let sender = self.sender.clone();

        std::thread::spawn(move || {
            let result = Self::load_fonts_internal();

            let mut state = state_clone.lock().unwrap();
            match &result {
                FontLoadState::Loaded(data) => {
                    log::trace!(
                        "Font loading completed: primary={} bytes, icon={}",
                        data.primary.len(),
                        data.icon.is_some()
                    );
                }
                FontLoadState::Failed(e) => {
                    log::error!("Font loading failed: {}", e);
                }
                _ => {}
            }
            *state = result.clone();
            let _ = sender.send(result);
        });
    }

    /// 获取当前加载状态
    pub fn state(&self) -> FontLoadState {
        self.state.lock().unwrap().clone()
    }

    /// 尝试获取已加载的字体数据 (非阻塞)
    pub fn try_get(&self) -> Option<FontData> {
        if let Ok(state) = self.receiver.try_recv() {
            match state {
                FontLoadState::Loaded(data) => Some(data),
                _ => None,
            }
        } else {
            match &*self.state.lock().unwrap() {
                FontLoadState::Loaded(data) => Some(data.clone()),
                _ => None,
            }
        }
    }

    /// 等待加载完成 (阻塞)
    pub fn wait(&self) -> FontData {
        match self.receiver.recv() {
            Ok(FontLoadState::Loaded(data)) => data,
            Ok(FontLoadState::Failed(e)) => panic!("Font loading failed: {}", e),
            Ok(_) => panic!("Font loading in unexpected state"),
            Err(e) => panic!("Font loading channel error: {}", e),
        }
    }

    /// 内部字体加载逻辑
    fn load_fonts_internal() -> FontLoadState {
        // 加载主字体
        let primary = match Self::load_primary_font() {
            Some(data) => data,
            None => return FontLoadState::Failed("No primary font found".to_string()),
        };

        // 加载CJK回退字体 (后台静默加载，失败不影响)
        let cjk_sc = Self::load_cjk_font(&[
            "C:\\Windows\\Fonts\\msyh.ttc",    // 微软雅黑
            "C:\\Windows\\Fonts\\msyhbd.ttc",   // 微软雅黑粗体
            "C:\\Windows\\Fonts\\simhei.ttf",   // 黑体
            "C:\\Windows\\Fonts\\simsun.ttc",   // 宋体
        ]);

        let cjk_tc = Self::load_cjk_font(&[
            "C:\\Windows\\Fonts\\msjh.ttc",     // 微软正黑
            "C:\\Windows\\Fonts\\msjhbd.ttc",   // 微软正黑粗体
        ]);

        let cjk_jp = Self::load_cjk_font(&[
            "C:\\Windows\\Fonts\\YuGothR.ttc",  // 游ゴシック
            "C:\\Windows\\Fonts\\msgothic.ttc",  // ゴシック
        ]);

        let cjk_kr = Self::load_cjk_font(&[
            "C:\\Windows\\Fonts\\malgun.ttf",   //맑은 고딕
            "C:\\Windows\\Fonts\\malgunbd.ttf", //맑은 고딕 Bold
        ]);

        // 加载图标字体 (Nerd Font)
        let icon = Self::load_icon_font();

        FontLoadState::Loaded(FontData {
            primary,
            cjk_sc,
            cjk_tc,
            cjk_jp,
            cjk_kr,
            icon,
        })
    }

    /// 加载主字体 (Segoe UI 优先)
    fn load_primary_font() -> Option<Vec<u8>> {
        let primary_paths = [
            "C:\\Windows\\Fonts\\segoeui.ttf",    // Segoe UI
            "C:\\Windows\\Fonts\\seguisb.ttf",    // Segoe UI Semibold
            "C:\\Windows\\Fonts\\arial.ttf",       // Arial
            "C:\\Windows\\Fonts\\tahoma.ttf",      // Tahoma
        ];

        for path in &primary_paths {
            if let Ok(data) = std::fs::read(path) {
                log::trace!("Loaded primary font from: {}", path);
                return Some(data);
            }
        }

        // 尝试使用系统字体
        log::warn!("No standard primary font found, trying system fonts");
        Self::find_system_font()
    }

    /// 查找系统字体
    fn find_system_font() -> Option<Vec<u8>> {
        let fonts_dir = std::path::Path::new("C:\\Windows\\Fonts");
        if let Ok(entries) = std::fs::read_dir(fonts_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    let ext_lower = ext.to_string_lossy().to_lowercase();
                    if ext_lower == "ttf" || ext_lower == "ttc" {
                        if let Ok(data) = std::fs::read(&path) {
                            log::trace!("Found system font: {}", path.display());
                            return Some(data);
                        }
                    }
                }
            }
        }
        None
    }

    /// 加载CJK字体
    fn load_cjk_font(paths: &[&str]) -> Option<Vec<u8>> {
        for path in paths {
            if let Ok(data) = std::fs::read(path) {
                log::trace!("Loaded CJK font from: {}", path);
                return Some(data);
            }
        }
        None
    }

    /// 加载图标字体 (Nerd Font)
    fn load_icon_font() -> Option<Vec<u8>> {
        let icon_paths = [
            "C:\\Windows\\Fonts\\JetBrainsMonoNerdFont-Regular.ttf",
            "C:\\Windows\\Fonts\\JetBrainsMonoNerdFontMono-Regular.ttf",
            "C:\\Windows\\Fonts\\JetBrainsMonoNerdFontPropo-Regular.ttf",
        ];

        for path in &icon_paths {
            if let Ok(data) = std::fs::read(path) {
                log::trace!("Loaded icon font from: {}", path);
                return Some(data);
            }
        }

        log::warn!("No Nerd Font found, icons may not render correctly");
        None
    }
}

impl Default for AsyncFontLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_loader_new() {
        let loader = AsyncFontLoader::new();
        assert_eq!(loader.state(), FontLoadState::NotStarted);
    }

    #[test]
    fn test_load_primary_font() {
        let result = AsyncFontLoader::load_primary_font();
        // 在Windows上应该能找到字体
        assert!(result.is_some());
    }
}

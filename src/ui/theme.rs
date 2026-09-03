/// 主题系统 - 参考 Tessoa 的 10+ 内置主题和自定义配色
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn from_hex(hex: u32) -> Self {
        let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
        let b = (hex & 0xFF) as f32 / 255.0;
        Self { r, g, b, a: 1.0 }
    }

    pub fn to_u32(&self) -> u32 {
        let r = (self.r * 255.0) as u32;
        let g = (self.g * 255.0) as u32;
        let b = (self.b * 255.0) as u32;
        let a = (self.a * 255.0) as u32;
        (a << 24) | (r << 16) | (g << 8) | b
    }

    /// 调整亮度
    pub fn adjust_brightness(&self, factor: f32) -> Self {
        Self {
            r: (self.r * factor).min(1.0),
            g: (self.g * factor).min(1.0),
            b: (self.b * factor).min(1.0),
            a: self.a,
        }
    }

    /// 混合两种颜色
    pub fn mix(&self, other: &Color, t: f32) -> Self {
        Self {
            r: self.r * (1.0 - t) + other.r * t,
            g: self.g * (1.0 - t) + other.g * t,
            b: self.b * (1.0 - t) + other.b * t,
            a: self.a,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub background: Color,
    pub surface: Color,
    pub surface_variant: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_disabled: Color,
    pub primary: Color,
    pub primary_light: Color,
    pub primary_dark: Color,
    pub accent: Color,
    pub accent_light: Color,
    pub hover: Color,
    pub pressed: Color,
    pub focused: Color,
    pub selected: Color,
    pub border: Color,
    pub border_light: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    pub folder: Color,
    pub file: Color,
    pub image: Color,
    pub video: Color,
    pub audio: Color,
    pub document: Color,
    pub archive: Color,
    pub code: Color,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self::dark()
    }
}

impl ThemeColors {
    pub fn dark() -> Self {
        Self {
            background: Color::from_hex(0x1E1E1E),
            surface: Color::from_hex(0x2D2D2D),
            surface_variant: Color::from_hex(0x3D3D3D),
            text_primary: Color::from_hex(0xFFFFFF),
            text_secondary: Color::from_hex(0xB0B0B0),
            text_disabled: Color::from_hex(0x606060),
            primary: Color::from_hex(0x4A90D9),
            primary_light: Color::from_hex(0x6BA3E0),
            primary_dark: Color::from_hex(0x3A7BC8),
            accent: Color::from_hex(0x9B59B6),
            accent_light: Color::from_hex(0xBB77D6),
            hover: Color::from_hex(0x3A3A3A),
            pressed: Color::from_hex(0x4A4A4A),
            focused: Color::from_hex(0x4A90D9),
            selected: Color::from_hex(0x4A90D9),
            border: Color::from_hex(0x404040),
            border_light: Color::from_hex(0x505050),
            success: Color::from_hex(0x2ECC71),
            warning: Color::from_hex(0xF39C12),
            error: Color::from_hex(0xE74C3C),
            info: Color::from_hex(0x3498DB),
            folder: Color::from_hex(0xF1C40F),
            file: Color::from_hex(0x95A5A6),
            image: Color::from_hex(0xE74C3C),
            video: Color::from_hex(0x9B59B6),
            audio: Color::from_hex(0x3498DB),
            document: Color::from_hex(0x2ECC71),
            archive: Color::from_hex(0xF39C12),
            code: Color::from_hex(0x1ABC9C),
        }
    }

    pub fn light() -> Self {
        Self {
            background: Color::from_hex(0xF9F9F9),
            surface: Color::from_hex(0xFFFFFF),
            surface_variant: Color::from_hex(0xF3F3F3),
            text_primary: Color::from_hex(0x1A1A1A),
            text_secondary: Color::from_hex(0x616161),
            text_disabled: Color::from_hex(0x9E9E9E),
            primary: Color::from_hex(0x0078D4),
            primary_light: Color::from_hex(0xE8F4FD),
            primary_dark: Color::from_hex(0x005A9E),
            accent: Color::from_hex(0x0078D4),
            accent_light: Color::from_hex(0xE8F4FD),
            hover: Color::from_hex(0xE8E8E8),
            pressed: Color::from_hex(0xD0D0D0),
            focused: Color::from_hex(0x0078D4),
            selected: Color::from_hex(0x0078D4),
            border: Color::from_hex(0xE5E5E5),
            border_light: Color::from_hex(0xD1D1D1),
            success: Color::from_hex(0x0F7B0F),
            warning: Color::from_hex(0x9D5D00),
            error: Color::from_hex(0xC42B1C),
            info: Color::from_hex(0x0078D4),
            folder: Color::from_hex(0xFFC107),
            file: Color::from_hex(0x9E9E9E),
            image: Color::from_hex(0xC42B1C),
            video: Color::from_hex(0x9B59B6),
            audio: Color::from_hex(0x0078D4),
            document: Color::from_hex(0x0F7B0F),
            archive: Color::from_hex(0x9D5D00),
            code: Color::from_hex(0x00BCD4),
        }
    }

    /// 从主色调生成完整主题
    pub fn from_primary(primary: Color) -> Self {
        let mut colors = Self::dark();
        colors.primary = primary.clone();
        colors.primary_light = primary.adjust_brightness(1.2);
        colors.primary_dark = primary.adjust_brightness(0.8);
        colors.accent = primary.clone();
        colors.accent_light = primary.adjust_brightness(1.2);
        colors.focused = primary.clone();
        colors.selected = primary.clone();
        colors
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spacing {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
}

impl Default for Spacing {
    fn default() -> Self {
        Self {
            xs: 2.0,
            sm: 4.0,
            md: 8.0,
            lg: 16.0,
            xl: 24.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

impl ThemeMode {
    pub fn display_name(&self) -> &str {
        match self {
            ThemeMode::Light => "Light",
            ThemeMode::Dark => "Dark",
            ThemeMode::System => "System",
        }
    }
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self::System
    }
}

/// 内置主题类型 - 对齐 Tessoa 官方 10 套主题
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuiltInTheme {
    /// 青铜雪 - 深色主题（出厂深色）
    BronzeSnow,
    /// 海盐蓝 - 浅色主题（出厂浅色）
    SeaSaltBlue,
    /// 海图蓝 - 蓝色系
    ChartBlue,
    /// 绯樱 - 粉色系
    CrimsonSakura,
    /// 碧波 - 青色系
    AzureWave,
    /// 黛蓝 - 深蓝系
    IndigoBlue,
    /// 暖砂 - 暖色系
    WarmSand,
    /// 曜黑 - 纯黑系
    ObsidianBlack,
    /// 暮紫 - 紫色系
    TwilightPurple,
    /// 晴空 - 天蓝系
    ClearSky,
}

impl BuiltInTheme {
    /// 获取显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::BronzeSnow => "青铜雪",
            Self::SeaSaltBlue => "海盐蓝",
            Self::ChartBlue => "海图蓝",
            Self::CrimsonSakura => "绯樱",
            Self::AzureWave => "碧波",
            Self::IndigoBlue => "黛蓝",
            Self::WarmSand => "暖砂",
            Self::ObsidianBlack => "曜黑",
            Self::TwilightPurple => "暮紫",
            Self::ClearSky => "晴空",
        }
    }

    /// 从索引获取
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Self::BronzeSnow,
            1 => Self::SeaSaltBlue,
            2 => Self::ChartBlue,
            3 => Self::CrimsonSakura,
            4 => Self::AzureWave,
            5 => Self::IndigoBlue,
            6 => Self::WarmSand,
            7 => Self::ObsidianBlack,
            8 => Self::TwilightPurple,
            9 => Self::ClearSky,
            _ => Self::BronzeSnow,
        }
    }

    /// 转换为索引
    pub fn to_index(&self) -> usize {
        match self {
            Self::BronzeSnow => 0,
            Self::SeaSaltBlue => 1,
            Self::ChartBlue => 2,
            Self::CrimsonSakura => 3,
            Self::AzureWave => 4,
            Self::IndigoBlue => 5,
            Self::WarmSand => 6,
            Self::ObsidianBlack => 7,
            Self::TwilightPurple => 8,
            Self::ClearSky => 9,
        }
    }

    /// 获取所有内置主题
    pub fn all() -> Vec<Self> {
        vec![
            Self::BronzeSnow,
            Self::SeaSaltBlue,
            Self::ChartBlue,
            Self::CrimsonSakura,
            Self::AzureWave,
            Self::IndigoBlue,
            Self::WarmSand,
            Self::ObsidianBlack,
            Self::TwilightPurple,
            Self::ClearSky,
        ]
    }

    /// 是否是深色主题
    pub fn is_dark(&self) -> bool {
        matches!(self, Self::BronzeSnow | Self::ObsidianBlack | Self::IndigoBlue | Self::TwilightPurple)
    }
}

/// 自定义颜色方案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomColorScheme {
    pub name: String,
    pub colors: ThemeColors,
    pub is_dark: bool,
}

/// 主题管理器
#[derive(Debug, Clone)]
pub struct ThemeManager {
    mode: ThemeMode,
    current: Theme,
    custom_schemes: Vec<CustomColorScheme>,
    current_scheme_index: Option<usize>,
}

impl ThemeManager {
    pub fn new() -> Self {
        Self {
            mode: ThemeMode::default(),
            current: Theme::dark(),
            custom_schemes: Vec::new(),
            current_scheme_index: None,
        }
    }

    pub fn with_mode(mode: ThemeMode) -> Self {
        let mut manager = Self::new();
        manager.set_mode(mode);
        manager
    }

    pub fn mode(&self) -> ThemeMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: ThemeMode) {
        self.mode = mode;
        self.current = match mode {
            ThemeMode::Light => Theme::light(),
            ThemeMode::Dark => Theme::dark(),
            ThemeMode::System => Self::detect_system_theme(),
        };
    }

    pub fn theme(&self) -> &Theme {
        &self.current
    }

    pub fn toggle(&mut self) {
        self.set_mode(match self.mode {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::System => ThemeMode::Light,
        });
    }

    /// 设置内置主题
    pub fn set_builtin_theme(&mut self, theme: BuiltInTheme) {
        self.current = Theme::from_builtin(theme);
        self.current_scheme_index = None;
    }

    /// 添加自定义颜色方案
    pub fn add_custom_scheme(&mut self, scheme: CustomColorScheme) -> usize {
        let index = self.custom_schemes.len();
        self.custom_schemes.push(scheme);
        index
    }

    /// 设置自定义颜色方案
    pub fn set_custom_scheme(&mut self, index: usize) -> bool {
        if index < self.custom_schemes.len() {
            let scheme = self.custom_schemes[index].clone();
            self.current = Theme::from_colors(scheme.colors);
            self.current_scheme_index = Some(index);
            true
        } else {
            false
        }
    }

    /// 获取自定义颜色方案列表
    pub fn custom_schemes(&self) -> &[CustomColorScheme] {
        &self.custom_schemes
    }

    /// 获取当前方案索引
    pub fn current_scheme_index(&self) -> Option<usize> {
        self.current_scheme_index
    }

    /// 删除自定义颜色方案
    pub fn remove_custom_scheme(&mut self, index: usize) -> bool {
        if index < self.custom_schemes.len() {
            self.custom_schemes.remove(index);
            if self.current_scheme_index == Some(index) {
                self.current_scheme_index = None;
            } else if let Some(current) = self.current_scheme_index {
                if current > index {
                    self.current_scheme_index = Some(current - 1);
                }
            }
            true
        } else {
            false
        }
    }

    /// 导出颜色方案为 JSON
    pub fn export_scheme(&self, index: usize) -> Option<String> {
        self.custom_schemes.get(index).and_then(|scheme| {
            serde_json::to_string_pretty(scheme).ok()
        })
    }

    /// 从 JSON 导入颜色方案
    pub fn import_scheme(&mut self, json: &str) -> Result<usize, String> {
        let scheme: CustomColorScheme = serde_json::from_str(json)
            .map_err(|e| format!("JSON 解析失败: {}", e))?;
        Ok(self.add_custom_scheme(scheme))
    }

    /// 导出所有自定义方案
    pub fn export_all_schemes(&self) -> String {
        serde_json::to_string_pretty(&self.custom_schemes).unwrap_or_default()
    }

    /// 导入所有自定义方案
    pub fn import_all_schemes(&mut self, json: &str) -> Result<(), String> {
        let schemes: Vec<CustomColorScheme> = serde_json::from_str(json)
            .map_err(|e| format!("JSON 解析失败: {}", e))?;
        self.custom_schemes = schemes;
        self.current_scheme_index = None;
        Ok(())
    }

    fn detect_system_theme() -> Theme {
        #[cfg(target_os = "windows")]
        {
            if Self::is_windows_dark_mode() {
                Theme::dark()
            } else {
                Theme::light()
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            Theme::dark()
        }
    }

    #[cfg(target_os = "windows")]
    fn is_windows_dark_mode() -> bool {
        use std::process::Command;
        Command::new("reg")
            .args([
                "query",
                r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Themes\Personalize",
                "/v",
                "AppsUseLightTheme",
                "/t",
                "REG_DWORD",
            ])
            .output()
            .map(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.contains("0x0")
            })
            .unwrap_or(false)
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub colors: ThemeColors,
    pub spacing: Spacing,
    pub font_size: f32,
    pub line_height: f32,
    pub border_radius: f32,
    pub row_height: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            colors: ThemeColors::dark(),
            spacing: Spacing::default(),
            font_size: 14.0,
            line_height: 1.5,
            border_radius: 4.0,
            row_height: 36.0,
        }
    }

    pub fn light() -> Self {
        Self {
            colors: ThemeColors::light(),
            spacing: Spacing::default(),
            font_size: 14.0,
            line_height: 1.5,
            border_radius: 4.0,
            row_height: 36.0,
        }
    }

    /// 从内置主题创建 - 对齐 Tessoa 官方 10 套主题
    pub fn from_builtin(builtin: BuiltInTheme) -> Self {
        match builtin {
            BuiltInTheme::BronzeSnow => Self::bronze_snow(),
            BuiltInTheme::SeaSaltBlue => Self::sea_salt_blue(),
            BuiltInTheme::ChartBlue => Self::chart_blue(),
            BuiltInTheme::CrimsonSakura => Self::crimson_sakura(),
            BuiltInTheme::AzureWave => Self::azure_wave(),
            BuiltInTheme::IndigoBlue => Self::indigo_blue(),
            BuiltInTheme::WarmSand => Self::warm_sand(),
            BuiltInTheme::ObsidianBlack => Self::obsidian_black(),
            BuiltInTheme::TwilightPurple => Self::twilight_purple(),
            BuiltInTheme::ClearSky => Self::clear_sky(),
        }
    }

    /// 从主色调创建主题
    pub fn from_primary_color(primary: Color) -> Self {
        Self {
            colors: ThemeColors::from_primary(primary),
            spacing: Spacing::default(),
            font_size: 14.0,
            line_height: 1.5,
            border_radius: 4.0,
            row_height: 36.0,
        }
    }

    /// 从自定义颜色创建主题
    pub fn from_colors(colors: ThemeColors) -> Self {
        Self {
            colors,
            spacing: Spacing::default(),
            font_size: 14.0,
            line_height: 1.5,
            border_radius: 4.0,
            row_height: 36.0,
        }
    }

    /// 青铜雪 - 深色主题（出厂深色）
    /// 参考 Tessoa 官方配色：深沉的青铜色调
    pub fn bronze_snow() -> Self {
        Self {
            colors: ThemeColors {
                background: Color::from_hex(0x1A1A1E),      // 深灰黑
                surface: Color::from_hex(0x252528),          // 次深灰
                surface_variant: Color::from_hex(0x2E2E32),  // 第三深灰
                text_primary: Color::from_hex(0xE8E8EC),     // 浅灰白
                text_secondary: Color::from_hex(0xA0A0A8),   // 中灰
                text_disabled: Color::from_hex(0x606068),    // 暗灰
                primary: Color::from_hex(0x8B7355),          // 青铜色
                primary_light: Color::from_hex(0xA89070),    // 浅青铜
                primary_dark: Color::from_hex(0x6B5335),     // 深青铜
                accent: Color::from_hex(0xC9A96E),           // 金色强调
                accent_light: Color::from_hex(0xD4BA85),     // 浅金
                hover: Color::from_hex(0x353538),            // 悬停背景
                pressed: Color::from_hex(0x404045),          // 按下背景
                focused: Color::from_hex(0x8B7355),          // 聚焦边框
                selected: Color::from_hex(0x8B7355),         // 选中背景
                border: Color::from_hex(0x3A3A40),           // 边框
                border_light: Color::from_hex(0x4A4A50),     // 浅边框
                success: Color::from_hex(0x7BAE7F),          // 成功绿
                warning: Color::from_hex(0xD4A76A),          // 警告橙
                error: Color::from_hex(0xC9706E),            // 错误红
                info: Color::from_hex(0x7BA3C9),             // 信息蓝
                folder: Color::from_hex(0xC9A96E),           // 文件夹色
                file: Color::from_hex(0x909098),             // 文件色
                image: Color::from_hex(0xC9706E),            // 图片色
                video: Color::from_hex(0x9B8EC9),            // 视频色
                audio: Color::from_hex(0x7BA3C9),            // 音频色
                document: Color::from_hex(0x7BAE7F),         // 文档色
                archive: Color::from_hex(0xD4A76A),          // 压缩包色
                code: Color::from_hex(0x8BC9C9),             // 代码色
            },
            spacing: Spacing::default(),
            font_size: 14.0,
            line_height: 1.5,
            border_radius: 4.0,
            row_height: 36.0,
        }
    }

    /// 海盐蓝 - 浅色主题（出厂浅色）
    /// 参考 Tessoa 官方配色：清新的海盐蓝
    pub fn sea_salt_blue() -> Self {
        Self {
            colors: ThemeColors {
                background: Color::from_hex(0xF5F7FA),      // 浅灰蓝背景
                surface: Color::from_hex(0xFFFFFF),          // 白色表面
                surface_variant: Color::from_hex(0xEef1F5),  // 浅蓝灰
                text_primary: Color::from_hex(0x1A2332),     // 深蓝黑
                text_secondary: Color::from_hex(0x5A6577),   // 中蓝灰
                text_disabled: Color::from_hex(0xA0AAB8),    // 暗蓝灰
                primary: Color::from_hex(0x4A90B8),          // 海盐蓝
                primary_light: Color::from_hex(0x6AABD0),    // 浅海盐蓝
                primary_dark: Color::from_hex(0x3A7098),     // 深海盐蓝
                accent: Color::from_hex(0xE8913A),           // 橙色强调
                accent_light: Color::from_hex(0xF0A858),     // 浅橙
                hover: Color::from_hex(0xE8ECF0),            // 悬停背景
                pressed: Color::from_hex(0xD8DCE0),          // 按下背景
                focused: Color::from_hex(0x4A90B8),          // 聚焦边框
                selected: Color::from_hex(0x4A90B8),         // 选中背景
                border: Color::from_hex(0xD0D8E0),           // 边框
                border_light: Color::from_hex(0xE0E8F0),     // 浅边框
                success: Color::from_hex(0x4A9B6A),          // 成功绿
                warning: Color::from_hex(0xD49A3A),          // 警告橙
                error: Color::from_hex(0xC95A5A),            // 错误红
                info: Color::from_hex(0x4A90B8),             // 信息蓝
                folder: Color::from_hex(0xE8913A),           // 文件夹色
                file: Color::from_hex(0x8090A0),             // 文件色
                image: Color::from_hex(0xC95A5A),            // 图片色
                video: Color::from_hex(0x8B6AB8),            // 视频色
                audio: Color::from_hex(0x4A90B8),            // 音频色
                document: Color::from_hex(0x4A9B6A),         // 文档色
                archive: Color::from_hex(0xD49A3A),          // 压缩包色
                code: Color::from_hex(0x4AB8B8),             // 代码色
            },
            spacing: Spacing::default(),
            font_size: 14.0,
            line_height: 1.5,
            border_radius: 4.0,
            row_height: 36.0,
        }
    }

    /// 海图蓝 - 蓝色系
    pub fn chart_blue() -> Self {
        Self {
            colors: ThemeColors {
                background: Color::from_hex(0x1E2A3A),
                surface: Color::from_hex(0x283848),
                surface_variant: Color::from_hex(0x324858),
                text_primary: Color::from_hex(0xE0E8F0),
                text_secondary: Color::from_hex(0x90A0B0),
                text_disabled: Color::from_hex(0x506070),
                primary: Color::from_hex(0x3A7AB8),
                primary_light: Color::from_hex(0x5A9AD0),
                primary_dark: Color::from_hex(0x2A5A98),
                accent: Color::from_hex(0x4AB8D8),
                accent_light: Color::from_hex(0x6AD0E8),
                hover: Color::from_hex(0x304050),
                pressed: Color::from_hex(0x405060),
                focused: Color::from_hex(0x3A7AB8),
                selected: Color::from_hex(0x3A7AB8),
                border: Color::from_hex(0x3A4A5A),
                border_light: Color::from_hex(0x4A5A6A),
                success: Color::from_hex(0x5AB878),
                warning: Color::from_hex(0xD8A848),
                error: Color::from_hex(0xD86868),
                info: Color::from_hex(0x4AB8D8),
                folder: Color::from_hex(0x4AB8D8),
                file: Color::from_hex(0x8098A8),
                image: Color::from_hex(0xD86868),
                video: Color::from_hex(0x9878C8),
                audio: Color::from_hex(0x4AB8D8),
                document: Color::from_hex(0x5AB878),
                archive: Color::from_hex(0xD8A848),
                code: Color::from_hex(0x48C8C8),
            },
            spacing: Spacing::default(),
            font_size: 14.0,
            line_height: 1.5,
            border_radius: 4.0,
            row_height: 36.0,
        }
    }

    /// 绯樱 - 粉色系
    pub fn crimson_sakura() -> Self {
        Self {
            colors: ThemeColors {
                background: Color::from_hex(0x2A1A20),
                surface: Color::from_hex(0x3A2830),
                surface_variant: Color::from_hex(0x4A3840),
                text_primary: Color::from_hex(0xF0E0E8),
                text_secondary: Color::from_hex(0xB09098),
                text_disabled: Color::from_hex(0x604850),
                primary: Color::from_hex(0xC85A78),
                primary_light: Color::from_hex(0xD87898),
                primary_dark: Color::from_hex(0xA83A58),
                accent: Color::from_hex(0xE88AA0),
                accent_light: Color::from_hex(0xF0A8B8),
                hover: Color::from_hex(0x403038),
                pressed: Color::from_hex(0x504048),
                focused: Color::from_hex(0xC85A78),
                selected: Color::from_hex(0xC85A78),
                border: Color::from_hex(0x4A3840),
                border_light: Color::from_hex(0x5A4850),
                success: Color::from_hex(0x78B878),
                warning: Color::from_hex(0xD8A858),
                error: Color::from_hex(0xE86878),
                info: Color::from_hex(0x78A8D8),
                folder: Color::from_hex(0xE88AA0),
                file: Color::from_hex(0x908088),
                image: Color::from_hex(0xE86878),
                video: Color::from_hex(0xA878C8),
                audio: Color::from_hex(0x78A8D8),
                document: Color::from_hex(0x78B878),
                archive: Color::from_hex(0xD8A858),
                code: Color::from_hex(0x78C8C8),
            },
            spacing: Spacing::default(),
            font_size: 14.0,
            line_height: 1.5,
            border_radius: 4.0,
            row_height: 36.0,
        }
    }

    /// 碧波 - 青色系
    pub fn azure_wave() -> Self {
        Self {
            colors: ThemeColors {
                background: Color::from_hex(0x1A2A2A),
                surface: Color::from_hex(0x283838),
                surface_variant: Color::from_hex(0x324848),
                text_primary: Color::from_hex(0xE0F0F0),
                text_secondary: Color::from_hex(0x90B0B0),
                text_disabled: Color::from_hex(0x506868),
                primary: Color::from_hex(0x3AA8A8),
                primary_light: Color::from_hex(0x58C0C0),
                primary_dark: Color::from_hex(0x288888),
                accent: Color::from_hex(0x48D8D8),
                accent_light: Color::from_hex(0x68E8E8),
                hover: Color::from_hex(0x304040),
                pressed: Color::from_hex(0x405050),
                focused: Color::from_hex(0x3AA8A8),
                selected: Color::from_hex(0x3AA8A8),
                border: Color::from_hex(0x3A4A4A),
                border_light: Color::from_hex(0x4A5A5A),
                success: Color::from_hex(0x58C878),
                warning: Color::from_hex(0xD8B848),
                error: Color::from_hex(0xD86878),
                info: Color::from_hex(0x48D8D8),
                folder: Color::from_hex(0x48D8D8),
                file: Color::from_hex(0x80A0A0),
                image: Color::from_hex(0xD86878),
                video: Color::from_hex(0x9888C8),
                audio: Color::from_hex(0x48D8D8),
                document: Color::from_hex(0x58C878),
                archive: Color::from_hex(0xD8B848),
                code: Color::from_hex(0x48E8E8),
            },
            spacing: Spacing::default(),
            font_size: 14.0,
            line_height: 1.5,
            border_radius: 4.0,
            row_height: 36.0,
        }
    }

    /// 黛蓝 - 深蓝系
    pub fn indigo_blue() -> Self {
        Self {
            colors: ThemeColors {
                background: Color::from_hex(0x1A1A2E),
                surface: Color::from_hex(0x28283E),
                surface_variant: Color::from_hex(0x32324E),
                text_primary: Color::from_hex(0xE0E0F0),
                text_secondary: Color::from_hex(0x9090B0),
                text_disabled: Color::from_hex(0x505068),
                primary: Color::from_hex(0x5858B8),
                primary_light: Color::from_hex(0x7878D0),
                primary_dark: Color::from_hex(0x3838A0),
                accent: Color::from_hex(0x8888E8),
                accent_light: Color::from_hex(0xA8A8F0),
                hover: Color::from_hex(0x303048),
                pressed: Color::from_hex(0x404058),
                focused: Color::from_hex(0x5858B8),
                selected: Color::from_hex(0x5858B8),
                border: Color::from_hex(0x3A3A58),
                border_light: Color::from_hex(0x4A4A68),
                success: Color::from_hex(0x58B878),
                warning: Color::from_hex(0xD8A858),
                error: Color::from_hex(0xD86878),
                info: Color::from_hex(0x78A8E8),
                folder: Color::from_hex(0x8888E8),
                file: Color::from_hex(0x8888A0),
                image: Color::from_hex(0xD86878),
                video: Color::from_hex(0xA878D8),
                audio: Color::from_hex(0x78A8E8),
                document: Color::from_hex(0x58B878),
                archive: Color::from_hex(0xD8A858),
                code: Color::from_hex(0x78C8E8),
            },
            spacing: Spacing::default(),
            font_size: 14.0,
            line_height: 1.5,
            border_radius: 4.0,
            row_height: 36.0,
        }
    }

    /// 暖砂 - 暖色系
    pub fn warm_sand() -> Self {
        Self {
            colors: ThemeColors {
                background: Color::from_hex(0x2A2420),
                surface: Color::from_hex(0x3A3430),
                surface_variant: Color::from_hex(0x4A4440),
                text_primary: Color::from_hex(0xF0E8E0),
                text_secondary: Color::from_hex(0xB0A898),
                text_disabled: Color::from_hex(0x605848),
                primary: Color::from_hex(0xC89858),
                primary_light: Color::from_hex(0xD8B078),
                primary_dark: Color::from_hex(0xA87838),
                accent: Color::from_hex(0xE8A848),
                accent_light: Color::from_hex(0xF0C068),
                hover: Color::from_hex(0x403830),
                pressed: Color::from_hex(0x504840),
                focused: Color::from_hex(0xC89858),
                selected: Color::from_hex(0xC89858),
                border: Color::from_hex(0x4A4440),
                border_light: Color::from_hex(0x5A5450),
                success: Color::from_hex(0x78B868),
                warning: Color::from_hex(0xE8B848),
                error: Color::from_hex(0xD87868),
                info: Color::from_hex(0x78A8C8),
                folder: Color::from_hex(0xE8A848),
                file: Color::from_hex(0x908880),
                image: Color::from_hex(0xD87868),
                video: Color::from_hex(0xA888C8),
                audio: Color::from_hex(0x78A8C8),
                document: Color::from_hex(0x78B868),
                archive: Color::from_hex(0xE8B848),
                code: Color::from_hex(0x78C8B8),
            },
            spacing: Spacing::default(),
            font_size: 14.0,
            line_height: 1.5,
            border_radius: 4.0,
            row_height: 36.0,
        }
    }

    /// 曜黑 - 纯黑系
    pub fn obsidian_black() -> Self {
        Self {
            colors: ThemeColors {
                background: Color::from_hex(0x0A0A0A),
                surface: Color::from_hex(0x141414),
                surface_variant: Color::from_hex(0x1E1E1E),
                text_primary: Color::from_hex(0xF0F0F0),
                text_secondary: Color::from_hex(0xA0A0A0),
                text_disabled: Color::from_hex(0x505050),
                primary: Color::from_hex(0x606060),
                primary_light: Color::from_hex(0x808080),
                primary_dark: Color::from_hex(0x404040),
                accent: Color::from_hex(0x909090),
                accent_light: Color::from_hex(0xB0B0B0),
                hover: Color::from_hex(0x1A1A1A),
                pressed: Color::from_hex(0x242424),
                focused: Color::from_hex(0x606060),
                selected: Color::from_hex(0x606060),
                border: Color::from_hex(0x2A2A2A),
                border_light: Color::from_hex(0x3A3A3A),
                success: Color::from_hex(0x60A060),
                warning: Color::from_hex(0xC0A040),
                error: Color::from_hex(0xC06060),
                info: Color::from_hex(0x6080C0),
                folder: Color::from_hex(0x909090),
                file: Color::from_hex(0x707070),
                image: Color::from_hex(0xC06060),
                video: Color::from_hex(0x9070B0),
                audio: Color::from_hex(0x6080C0),
                document: Color::from_hex(0x60A060),
                archive: Color::from_hex(0xC0A040),
                code: Color::from_hex(0x60B0B0),
            },
            spacing: Spacing::default(),
            font_size: 14.0,
            line_height: 1.5,
            border_radius: 4.0,
            row_height: 36.0,
        }
    }

    /// 暮紫 - 紫色系
    pub fn twilight_purple() -> Self {
        Self {
            colors: ThemeColors {
                background: Color::from_hex(0x1E1A2A),
                surface: Color::from_hex(0x2E2838),
                surface_variant: Color::from_hex(0x3E3848),
                text_primary: Color::from_hex(0xF0E8F8),
                text_secondary: Color::from_hex(0xB0A0C0),
                text_disabled: Color::from_hex(0x605070),
                primary: Color::from_hex(0x8868B8),
                primary_light: Color::from_hex(0xA888D0),
                primary_dark: Color::from_hex(0x6848A0),
                accent: Color::from_hex(0xB888E8),
                accent_light: Color::from_hex(0xD0A8F0),
                hover: Color::from_hex(0x282038),
                pressed: Color::from_hex(0x383048),
                focused: Color::from_hex(0x8868B8),
                selected: Color::from_hex(0x8868B8),
                border: Color::from_hex(0x3A3048),
                border_light: Color::from_hex(0x4A4058),
                success: Color::from_hex(0x68B878),
                warning: Color::from_hex(0xD8A858),
                error: Color::from_hex(0xD86888),
                info: Color::from_hex(0x7898D8),
                folder: Color::from_hex(0xB888E8),
                file: Color::from_hex(0x8880A0),
                image: Color::from_hex(0xD86888),
                video: Color::from_hex(0xA878D8),
                audio: Color::from_hex(0x7898D8),
                document: Color::from_hex(0x68B878),
                archive: Color::from_hex(0xD8A858),
                code: Color::from_hex(0x78C8D8),
            },
            spacing: Spacing::default(),
            font_size: 14.0,
            line_height: 1.5,
            border_radius: 4.0,
            row_height: 36.0,
        }
    }

    /// 晴空 - 天蓝系
    pub fn clear_sky() -> Self {
        Self {
            colors: ThemeColors {
                background: Color::from_hex(0x1A2A38),
                surface: Color::from_hex(0x283848),
                surface_variant: Color::from_hex(0x324858),
                text_primary: Color::from_hex(0xE0F0FF),
                text_secondary: Color::from_hex(0x90B0C8),
                text_disabled: Color::from_hex(0x506878),
                primary: Color::from_hex(0x4898D8),
                primary_light: Color::from_hex(0x68B0E8),
                primary_dark: Color::from_hex(0x2878B8),
                accent: Color::from_hex(0x58C8F8),
                accent_light: Color::from_hex(0x78D8FF),
                hover: Color::from_hex(0x203848),
                pressed: Color::from_hex(0x304858),
                focused: Color::from_hex(0x4898D8),
                selected: Color::from_hex(0x4898D8),
                border: Color::from_hex(0x304858),
                border_light: Color::from_hex(0x405868),
                success: Color::from_hex(0x58C878),
                warning: Color::from_hex(0xD8B848),
                error: Color::from_hex(0xD86878),
                info: Color::from_hex(0x58C8F8),
                folder: Color::from_hex(0x58C8F8),
                file: Color::from_hex(0x8098B0),
                image: Color::from_hex(0xD86878),
                video: Color::from_hex(0x9888D8),
                audio: Color::from_hex(0x58C8F8),
                document: Color::from_hex(0x58C878),
                archive: Color::from_hex(0xD8B848),
                code: Color::from_hex(0x58D8E8),
            },
            spacing: Spacing::default(),
            font_size: 14.0,
            line_height: 1.5,
            border_radius: 4.0,
            row_height: 36.0,
        }
    }

    pub fn primary(&self) -> &Color {
        &self.colors.primary
    }

    pub fn primary_light(&self) -> &Color {
        &self.colors.primary_light
    }

    pub fn primary_dark(&self) -> &Color {
        &self.colors.primary_dark
    }

    pub fn background(&self) -> &Color {
        &self.colors.background
    }

    pub fn surface(&self) -> &Color {
        &self.colors.surface
    }

    pub fn surface_variant(&self) -> &Color {
        &self.colors.surface_variant
    }

    pub fn text_primary(&self) -> &Color {
        &self.colors.text_primary
    }

    pub fn text_secondary(&self) -> &Color {
        &self.colors.text_secondary
    }

    pub fn hover(&self) -> &Color {
        &self.colors.hover
    }

    pub fn pressed(&self) -> &Color {
        &self.colors.pressed
    }

    pub fn focused(&self) -> &Color {
        &self.colors.focused
    }

    pub fn selected(&self) -> &Color {
        &self.colors.selected
    }

    pub fn border(&self) -> &Color {
        &self.colors.border
    }

    pub fn border_light(&self) -> &Color {
        &self.colors.border_light
    }

    pub fn success(&self) -> &Color {
        &self.colors.success
    }

    pub fn warning(&self) -> &Color {
        &self.colors.warning
    }

    pub fn error(&self) -> &Color {
        &self.colors.error
    }

    pub fn info(&self) -> &Color {
        &self.colors.info
    }

    pub fn folder(&self) -> &Color {
        &self.colors.folder
    }

    pub fn file(&self) -> &Color {
        &self.colors.file
    }

    pub fn to_u32(&self) -> u32 {
        self.colors.primary.to_u32()
    }

    pub fn on_surface(&self) -> &Color {
        &self.colors.text_primary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_new() {
        let color = Color::new(0.5, 0.6, 0.7, 0.8);
        assert_eq!(color.r, 0.5);
        assert_eq!(color.g, 0.6);
        assert_eq!(color.b, 0.7);
        assert_eq!(color.a, 0.8);
    }

    #[test]
    fn test_color_rgb() {
        let color = Color::rgb(1.0, 0.0, 0.0);
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.0);
        assert_eq!(color.b, 0.0);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn test_color_from_hex() {
        let color = Color::from_hex(0xFF0000); // Red
        assert!(color.r > 0.99);
        assert!(color.g < 0.01);
        assert!(color.b < 0.01);
    }

    #[test]
    fn test_color_to_u32() {
        let color = Color::rgb(1.0, 0.0, 0.0); // Red
        let u32_color = color.to_u32();
        assert_eq!(u32_color, 0xFFFF0000);
    }

    #[test]
    fn test_color_adjust_brightness() {
        let color = Color::rgb(0.5, 0.5, 0.5);
        let brighter = color.adjust_brightness(1.5);
        assert!(brighter.r > color.r);
        let darker = color.adjust_brightness(0.5);
        assert!(darker.r < color.r);
    }

    #[test]
    fn test_color_mix() {
        let red = Color::rgb(1.0, 0.0, 0.0);
        let blue = Color::rgb(0.0, 0.0, 1.0);
        let purple = red.mix(&blue, 0.5);
        assert!(purple.r > 0.4 && purple.r < 0.6);
        assert!(purple.b > 0.4 && purple.b < 0.6);
    }

    #[test]
    fn test_theme_colors_dark() {
        let colors = ThemeColors::dark();
        assert!(colors.background.r < 0.5); // Dark background
    }

    #[test]
    fn test_theme_colors_light() {
        let colors = ThemeColors::light();
        assert!(colors.background.r > 0.5); // Light background
    }

    #[test]
    fn test_theme_dark() {
        let theme = Theme::dark();
        assert!(theme.colors.background.r < 0.5);
        assert_eq!(theme.font_size, 14.0);
        assert_eq!(theme.row_height, 36.0);
    }

    #[test]
    fn test_theme_light() {
        let theme = Theme::light();
        assert!(theme.colors.background.r > 0.5);
        assert_eq!(theme.font_size, 14.0);
    }

    #[test]
    fn test_theme_primary() {
        let theme = Theme::dark();
        let primary = theme.primary();
        assert!(primary.r > 0.0);
    }

    #[test]
    fn test_theme_manager_new() {
        let manager = ThemeManager::new();
        assert_eq!(manager.mode(), ThemeMode::System);
    }

    #[test]
    fn test_theme_manager_with_mode() {
        let manager = ThemeManager::with_mode(ThemeMode::Light);
        assert_eq!(manager.mode(), ThemeMode::Light);
        assert!(manager.theme().colors.background.r > 0.5);
    }

    #[test]
    fn test_theme_manager_set_mode() {
        let mut manager = ThemeManager::new();
        
        manager.set_mode(ThemeMode::Light);
        assert_eq!(manager.mode(), ThemeMode::Light);
        assert!(manager.theme().colors.background.r > 0.5);
        
        manager.set_mode(ThemeMode::Dark);
        assert_eq!(manager.mode(), ThemeMode::Dark);
        assert!(manager.theme().colors.background.r < 0.5);
    }

    #[test]
    fn test_theme_manager_toggle() {
        let mut manager = ThemeManager::with_mode(ThemeMode::System);
        
        manager.toggle();
        assert_eq!(manager.mode(), ThemeMode::Light);
        
        manager.toggle();
        assert_eq!(manager.mode(), ThemeMode::Dark);
        
        manager.toggle();
        assert_eq!(manager.mode(), ThemeMode::Light);
    }

    #[test]
    fn test_theme_mode_display_name() {
        assert_eq!(ThemeMode::Light.display_name(), "Light");
        assert_eq!(ThemeMode::Dark.display_name(), "Dark");
        assert_eq!(ThemeMode::System.display_name(), "System");
    }

    #[test]
    fn test_builtin_theme_all() {
        let all = BuiltInTheme::all();
        assert_eq!(all.len(), 10); // Tessoa 官方 10 套主题
    }

    #[test]
    fn test_builtin_theme_display_name() {
        assert_eq!(BuiltInTheme::BronzeSnow.display_name(), "青铜雪");
        assert_eq!(BuiltInTheme::SeaSaltBlue.display_name(), "海盐蓝");
        assert_eq!(BuiltInTheme::ClearSky.display_name(), "晴空");
    }

    #[test]
    fn test_builtin_theme_from_index() {
        assert_eq!(BuiltInTheme::from_index(0), BuiltInTheme::BronzeSnow);
        assert_eq!(BuiltInTheme::from_index(1), BuiltInTheme::SeaSaltBlue);
        assert_eq!(BuiltInTheme::from_index(9), BuiltInTheme::ClearSky);
        assert_eq!(BuiltInTheme::from_index(99), BuiltInTheme::BronzeSnow); // 默认
    }

    #[test]
    fn test_theme_from_builtin() {
        let theme = Theme::from_builtin(BuiltInTheme::BronzeSnow);
        assert!(theme.colors.background.r < 0.2); // 深色背景

        let theme = Theme::from_builtin(BuiltInTheme::SeaSaltBlue);
        assert!(theme.colors.background.r > 0.8); // 浅色背景
    }

    #[test]
    fn test_theme_manager_set_builtin() {
        let mut manager = ThemeManager::new();
        manager.set_builtin_theme(BuiltInTheme::ObsidianBlack);
        assert!(manager.theme().colors.background.r < 0.1); // 非常深
    }

    #[test]
    fn test_theme_manager_custom_schemes() {
        let mut manager = ThemeManager::new();
        
        let scheme = CustomColorScheme {
            name: "Test".to_string(),
            colors: ThemeColors::dark(),
            is_dark: true,
        };
        
        let index = manager.add_custom_scheme(scheme);
        assert_eq!(index, 0);
        assert_eq!(manager.custom_schemes().len(), 1);
        
        assert!(manager.set_custom_scheme(0));
        assert!(!manager.set_custom_scheme(1)); // Invalid index
    }

    #[test]
    fn test_theme_manager_export_import() {
        let mut manager = ThemeManager::new();
        
        let scheme = CustomColorScheme {
            name: "Test Export".to_string(),
            colors: ThemeColors::dark(),
            is_dark: true,
        };
        
        let index = manager.add_custom_scheme(scheme);
        
        // Export
        let json = manager.export_scheme(index);
        assert!(json.is_some());
        
        // Import
        let imported = manager.import_scheme(&json.unwrap());
        assert!(imported.is_ok());
        assert_eq!(manager.custom_schemes().len(), 2);
    }

    #[test]
    fn test_theme_manager_remove_scheme() {
        let mut manager = ThemeManager::new();
        
        let scheme1 = CustomColorScheme {
            name: "First".to_string(),
            colors: ThemeColors::dark(),
            is_dark: true,
        };
        
        let scheme2 = CustomColorScheme {
            name: "Second".to_string(),
            colors: ThemeColors::light(),
            is_dark: false,
        };
        
        manager.add_custom_scheme(scheme1);
        manager.add_custom_scheme(scheme2);
        
        assert!(manager.set_custom_scheme(0));
        assert!(manager.remove_custom_scheme(0));
        assert_eq!(manager.custom_schemes().len(), 1);
        assert!(manager.current_scheme_index().is_none());
    }
}

/// 文字渲染设置 - 参考 Tessoa 的五层文字渲染配置
#[derive(Debug, Clone)]
pub struct TextRenderSettings {
    /// 亚像素定位：每个字按亚像素精度落位，修复字距忽宽忽窄
    pub subpixel_positioning: bool,
    /// LCD 亚像素抗锯齿：把水平分辨率当三倍用（假设 RGB 排列）
    pub lcd_subpixel_aa: bool,
    /// 字形对齐像素格：笔画主干对齐整像素边界
    pub glyph_hinting: GlyphHinting,
    /// 文字 Gamma：控制笔画边缘深浅（0.60-1.60）
    pub text_gamma: f32,
    /// 经典文字渲染引擎：Windows 专用，使用系统 GDI 渲染（默认开启）
    /// 参考 Tessoa 设置：关 / 跟随字体 / 强制
    pub classic_text_engine: ClassicTextEngine,
}

/// 经典文字渲染引擎设置 - Windows 专用
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassicTextEngine {
    /// 关闭：使用自定义渲染引擎
    Off,
    /// 跟随字体：字体指定使用 GDI 时才用经典引擎
    FollowFont,
    /// 强制：所有字体都使用经典 GDI 渲染
    Force,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphHinting {
    /// 不做对齐，字形最忠实
    Off,
    /// 按字体自己的规定办
    FollowFont,
    /// 无视字体的规定，任何字号都对齐
    Force,
}

impl Default for TextRenderSettings {
    fn default() -> Self {
        Self {
            subpixel_positioning: true,
            lcd_subpixel_aa: false,
            glyph_hinting: GlyphHinting::Off,
            text_gamma: 1.0,
            #[cfg(target_os = "windows")]
            classic_text_engine: ClassicTextEngine::FollowFont,
            #[cfg(not(target_os = "windows"))]
            classic_text_engine: ClassicTextEngine::Off,
        }
    }
}

impl TextRenderSettings {
    /// 验证 gamma 范围
    pub fn set_gamma(&mut self, gamma: f32) {
        self.text_gamma = gamma.clamp(0.60, 1.60);
    }

    /// 设置经典文字渲染引擎
    pub fn set_classic_text_engine(&mut self, engine: ClassicTextEngine) {
        self.classic_text_engine = engine;
    }

    /// 是否使用经典文字渲染引擎
    pub fn use_classic_engine(&self) -> bool {
        match self.classic_text_engine {
            ClassicTextEngine::Off => false,
            ClassicTextEngine::FollowFont => {
                // 实际实现中需要检查字体属性
                // 这里返回 false 作为默认
                false
            }
            ClassicTextEngine::Force => true,
        }
    }
}

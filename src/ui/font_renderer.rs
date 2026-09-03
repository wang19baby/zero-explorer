use ab_glyph::{FontVec, PxScale, GlyphId, Font, ScaleFont, Point};
use std::collections::HashMap;
use crate::ui::text_render_settings::{TextRenderSettings, GlyphHinting};

/// 子像素位置量化为 4 个桶（0, 0.25, 0.5, 0.75）
const SUBPIXEL_BINS: u8 = 4;

/// 缓存键：glyph_id + 子像素桶
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct GlyphCacheKey {
    pub glyph_id: GlyphId,
    pub subpixel_bin: u8,
}

/// 预光栅化的字形数据
#[derive(Clone, Debug)]
pub struct RasterizedGlyph {
    pub width: u32,
    pub height: u32,
    pub bearing_x: f32,
    pub bearing_y: f32,
    pub advance: f32,
    pub pixels: Vec<u8>,
    /// LCD 子像素抗锯齿：分别存储 R/G/B 通道的 coverage（如果启用）
    pub lcd_pixels: Option<Vec<u8>>,
}

/// 字体渲染器 - 负责字形光栅化和缓存
/// 支持多字体回退，用于渲染特殊符号（如文件图标）
pub struct FontRenderer {
    fonts: Vec<FontVec>,
    font_size: f32,
    glyph_cache: HashMap<GlyphCacheKey, RasterizedGlyph>,
    glyph_font_map: HashMap<GlyphId, usize>,
    pub settings: TextRenderSettings,
}

impl FontRenderer {
    pub fn new(font_data: Vec<u8>, font_size: f32) -> Result<Self, String> {
        let font = FontVec::try_from_vec(font_data)
            .map_err(|e| format!("Failed to load font: {}", e))?;
        
        let mut renderer = Self {
            fonts: vec![font],
            font_size,
            glyph_cache: HashMap::new(),
            glyph_font_map: HashMap::new(),
            settings: TextRenderSettings::default(),
        };
        
        renderer.pre_rasterize_ascii();
        
        Ok(renderer)
    }
    
    /// 添加回退字体（用于特殊符号，如 Segoe MDL2 Assets）
    pub fn add_fallback_font(&mut self, font_data: Vec<u8>) -> Result<(), String> {
        let font = FontVec::try_from_vec(font_data)
            .map_err(|e| format!("Failed to load fallback font: {}", e))?;
        self.fonts.push(font);
        Ok(())
    }
    
    /// 设置文字渲染设置
    pub fn set_settings(&mut self, settings: TextRenderSettings) {
        // 如果设置变化导致需要重新光栅化，清除缓存
        let settings_changed = self.settings.lcd_subpixel_aa != settings.lcd_subpixel_aa
            || self.settings.glyph_hinting != settings.glyph_hinting
            || (self.settings.text_gamma - settings.text_gamma).abs() > 0.01;
        
        self.settings = settings;
        
        if settings_changed {
            self.glyph_cache.clear();
            self.pre_rasterize_ascii();
        }
    }
    
    /// 将子像素偏移量化为桶索引
    fn subpixel_bin(offset: f32) -> u8 {
        let fractional = offset - offset.floor();
        ((fractional * SUBPIXEL_BINS as f32).round() as u8).min(SUBPIXEL_BINS - 1)
    }
    
    fn pre_rasterize_ascii(&mut self) {
        for c in 32u8..=126 {
            if let Some(ch) = char::from_u32(c as u32) {
                if ch == ' ' || ch == '\t' {
                    let glyph_id = Font::glyph_id(&self.fonts[0], ch);
                    let scale_font = self.fonts[0].as_scaled(PxScale::from(self.font_size));
                    let advance = ScaleFont::h_advance(&scale_font, glyph_id);
                    let rasterized = RasterizedGlyph {
                        width: 0,
                        height: 0,
                        bearing_x: 0.0,
                        bearing_y: 0.0,
                        advance,
                        pixels: vec![],
                        lcd_pixels: None,
                    };
                    self.glyph_font_map.insert(glyph_id, 0);
                    let key = GlyphCacheKey { glyph_id, subpixel_bin: 0 };
                    self.glyph_cache.insert(key, rasterized);
                } else {
                    self.rasterize_glyph_with_offset(ch, 0.0);
                }
            }
        }
    }
    
    /// 预光栅化 Nerd Font 图标字符
    pub fn pre_rasterize_mdl2_icons(&mut self) {
        let icon_chars = [
            '\u{ea83}', '\u{e7a8}', '\u{e781}', '\u{e8ca}',
            '\u{e73c}', '\u{e736}', '\u{e749}', '\u{e80b}',
            '\u{e8eb}', '\u{e771}', '\u{e779}', '\u{e626}',
            '\u{e739}', '\u{e73d}', '\u{e755}', '\u{e73e}',
            '\u{e7ac}',
        ];
        
        log::trace!("Pre-rasterizing {} Nerd Font icons", icon_chars.len());
        for ch in icon_chars {
            if let Some(glyph) = self.rasterize_glyph_with_offset(ch, 0.0) {
                log::trace!("Pre-rasterized Nerd Font icon U+{:04X}: {}x{}", ch as u32, glyph.width, glyph.height);
            } else {
                log::warn!("Failed to pre-rasterize Nerd Font icon U+{:04X}", ch as u32);
            }
        }
    }
    
    fn is_emoji(ch: char) -> bool {
        let cp = ch as u32;
        (0x1F000..=0x1FFFF).contains(&cp)
            || (0x2600..=0x27BF).contains(&cp)
            || (0xFE00..=0xFE0F).contains(&cp)
            || (0x200D..=0x200D).contains(&cp)
    }
    
    pub fn find_font_for_char(&self, ch: char) -> Option<(usize, GlyphId)> {
        log::trace!("Finding font for char '{}' (U+{:04X})", ch, ch as u32);
        
        let emoji = Self::is_emoji(ch);
        let main_font = &self.fonts[0];
        let glyph_id = Font::glyph_id(main_font, ch);
        
        if glyph_id.0 != 0 {
            if emoji {
                log::trace!("Found emoji glyph '{}' in main font (id={})", ch, glyph_id.0);
                return Some((0, glyph_id));
            }
            let glyph = glyph_id.with_scale_and_position(
                PxScale::from(self.font_size),
                Point { x: 0.0, y: 0.0 },
            );
            if main_font.outline_glyph(glyph).is_some() || ch == ' ' || ch == '\t' {
                log::trace!("Found glyph '{}' in main font", ch);
                return Some((0, glyph_id));
            }
        }
        
        for (i, font) in self.fonts.iter().enumerate().skip(1) {
            let glyph_id = Font::glyph_id(font, ch);
            if glyph_id.0 != 0 {
                if emoji {
                    log::trace!("Found emoji glyph '{}' in fallback font {} (id={})", ch, i, glyph_id.0);
                    return Some((i, glyph_id));
                }
                let glyph = glyph_id.with_scale_and_position(
                    PxScale::from(self.font_size),
                    Point { x: 0.0, y: 0.0 },
                );
                let has_outline = font.outline_glyph(glyph).is_some();
                log::trace!("Checking glyph '{}' in fallback font {}: glyph_id={}, has_outline={}", ch, i, glyph_id.0, has_outline);
                if has_outline || ch == ' ' || ch == '\t' {
                    log::trace!("Found glyph '{}' in fallback font {}", ch, i);
                    return Some((i, glyph_id));
                }
            }
        }
        
        log::trace!("Glyph '{}' not found in any font", ch);
        None
    }
    
    /// 光栅化字形，支持子像素偏移和 hinting
    fn rasterize_glyph_with_offset(&mut self, ch: char, subpixel_offset: f32) -> Option<&RasterizedGlyph> {
        let (font_idx, glyph_id) = self.find_font_for_char(ch)?;
        
        let bin = Self::subpixel_bin(subpixel_offset);
        let key = GlyphCacheKey { glyph_id, subpixel_bin: bin };
        
        if self.glyph_cache.contains_key(&key) {
            return self.glyph_cache.get(&key);
        }
        
        let font = &self.fonts[font_idx];
        let scale = PxScale::from(self.font_size);
        
        // 应用子像素偏移（水平方向）
        let x_offset = subpixel_offset;
        let glyph = glyph_id.with_scale_and_position(
            scale,
            Point { x: x_offset, y: 0.0 },
        );
        
        let scale_font = font.as_scaled(scale);
        let advance = ScaleFont::h_advance(&scale_font, glyph_id);
        
        match font.outline_glyph(glyph) {
            Some(outlined) => {
                let mut bounds = outlined.px_bounds();
                
                // 应用 glyph hinting：将 stem 对齐到像素边界
                if self.settings.glyph_hinting != GlyphHinting::Off {
                    let should_hint = match self.settings.glyph_hinting {
                        GlyphHinting::Force => true,
                        GlyphHinting::FollowFont => {
                            // 当字号 >= 某阈值时，字体通常不再要求 hinting
                            self.font_size < 20.0
                        }
                        GlyphHinting::Off => false,
                    };
                    
                    if should_hint {
                        // 将 bearing_x 对齐到最近的整像素
                        let rounded_x = bounds.min.x.round();
                        bounds.min.x = rounded_x;
                        bounds.max.x = rounded_x + (bounds.max.x - bounds.min.x).round();
                    }
                }
                
                let width = (bounds.width() as u32).max(1);
                let height = (bounds.height() as u32).max(1);
                
                let lcd = self.settings.lcd_subpixel_aa;
                let gamma = self.settings.text_gamma;
                
                if lcd {
                    // LCD 子像素抗锯齿：分别计算 R/G/B 通道的 coverage
                    let mut rgb_pixels = vec![0u8; (width * height * 4) as usize];
                    
                    outlined.draw(|x, y, coverage| {
                        if x < width && y < height {
                            let idx = ((y * width + x) * 4) as usize;
                            if idx + 3 < rgb_pixels.len() {
                                // 对于每个像素，计算 R/G/B 子像素的 coverage
                                // 子像素排列：R 在左 1/3，G 在中 1/3，B 在右 1/3
                                let pixel_x = x as f32 + 0.5;
                                
                                // 计算子像素覆盖
                                let r_coverage = compute_subpixel_coverage(pixel_x - 1.0/3.0, coverage);
                                let g_coverage = compute_subpixel_coverage(pixel_x, coverage);
                                let b_coverage = compute_subpixel_coverage(pixel_x + 1.0/3.0, coverage);
                                
                                // 应用 gamma
                                let r = apply_gamma(r_coverage, gamma);
                                let g = apply_gamma(g_coverage, gamma);
                                let b = apply_gamma(b_coverage, gamma);
                                
                                rgb_pixels[idx] = (r * 255.0) as u8;
                                rgb_pixels[idx + 1] = (g * 255.0) as u8;
                                rgb_pixels[idx + 2] = (b * 255.0) as u8;
                                rgb_pixels[idx + 3] = 255; // 完全不透明，颜色由子像素控制
                            }
                        }
                    });
                    
                    let rasterized = RasterizedGlyph {
                        width,
                        height,
                        bearing_x: bounds.min.x,
                        bearing_y: bounds.min.y,
                        advance,
                        pixels: rgb_pixels.clone(),
                        lcd_pixels: Some(rgb_pixels),
                    };
                    
                    self.glyph_font_map.insert(glyph_id, font_idx);
                    let key = GlyphCacheKey { glyph_id, subpixel_bin: bin };
                    self.glyph_cache.insert(key, rasterized);
                    self.glyph_cache.get(&GlyphCacheKey { glyph_id, subpixel_bin: bin })
                } else {
                    // 标准灰阶抗锯齿
                    let mut pixels = vec![0u8; (width * height * 4) as usize];
                    
                    outlined.draw(|x, y, coverage| {
                        if x < width && y < height {
                            let idx = ((y * width + x) * 4) as usize;
                            if idx + 3 < pixels.len() {
                                let alpha = apply_gamma(coverage, gamma);
                                pixels[idx] = 255;
                                pixels[idx + 1] = 255;
                                pixels[idx + 2] = 255;
                                pixels[idx + 3] = (alpha * 255.0) as u8;
                            }
                        }
                    });
                    
                    let rasterized = RasterizedGlyph {
                        width,
                        height,
                        bearing_x: bounds.min.x,
                        bearing_y: bounds.min.y,
                        advance,
                        pixels,
                        lcd_pixels: None,
                    };
                    
                    self.glyph_font_map.insert(glyph_id, font_idx);
                    let key = GlyphCacheKey { glyph_id, subpixel_bin: bin };
                    self.glyph_cache.insert(key, rasterized);
                    self.glyph_cache.get(&GlyphCacheKey { glyph_id, subpixel_bin: bin })
                }
            }
            None => {
                let placeholder_size = (self.font_size * 0.8) as u32;
                let rasterized = RasterizedGlyph {
                    width: placeholder_size,
                    height: placeholder_size,
                    bearing_x: 0.0,
                    bearing_y: -self.font_size * 0.8,
                    advance,
                    pixels: vec![],
                    lcd_pixels: None,
                };
                
                self.glyph_font_map.insert(glyph_id, font_idx);
                let key = GlyphCacheKey { glyph_id, subpixel_bin: bin };
                self.glyph_cache.insert(key, rasterized);
                self.glyph_cache.get(&GlyphCacheKey { glyph_id, subpixel_bin: bin })
            }
        }
    }
    
    /// 获取字形（使用子像素定位）
    pub fn get_glyph(&mut self, ch: char, subpixel_offset: f32) -> Option<&RasterizedGlyph> {
        let result = self.find_font_for_char(ch);
        if result.is_none() {
            log::warn!("Cannot find font for char '{}' (U+{:04X})", ch, ch as u32);
            return None;
        }
        
        if self.settings.subpixel_positioning {
            self.rasterize_glyph_with_offset(ch, subpixel_offset)
        } else {
            self.rasterize_glyph_with_offset(ch, 0.0)
        }
    }
    
    /// 获取字形（兼容旧接口，不使用子像素定位）
    pub fn get_glyph_compat(&mut self, ch: char) -> Option<&RasterizedGlyph> {
        self.get_glyph(ch, 0.0)
    }
    
    pub fn glyph_id(&self, ch: char) -> GlyphId {
        if let Some((_, glyph_id)) = self.find_font_for_char(ch) {
            glyph_id
        } else {
            Font::glyph_id(&self.fonts[0], ch)
        }
    }
    
    pub fn measure_text(&self, text: &str) -> f32 {
        let scale_font = self.fonts[0].as_scaled(PxScale::from(self.font_size));
        let mut width = 0.0;
        for ch in text.chars() {
            let glyph_id = Font::glyph_id(&self.fonts[0], ch);
            // 从任意子像素桶中获取 advance（advance 不随子像素位置变化）
            let key = GlyphCacheKey { glyph_id, subpixel_bin: 0 };
            if let Some(glyph) = self.glyph_cache.get(&key) {
                width += glyph.advance;
            } else {
                width += ScaleFont::h_advance(&scale_font, glyph_id);
            }
        }
        width
    }
    
    pub fn font_size(&self) -> f32 {
        self.font_size
    }
    
    pub fn line_height(&self) -> f32 {
        let scale_font = self.fonts[0].as_scaled(PxScale::from(self.font_size));
        scale_font.height()
    }
    
    pub fn ascent(&self) -> f32 {
        let scale_font = self.fonts[0].as_scaled(PxScale::from(self.font_size));
        scale_font.ascent()
    }
    
    /// 获取字形缓存（按 glyph_id 查找，优先返回 bin=0）
    pub fn glyph_cache(&self) -> &HashMap<GlyphCacheKey, RasterizedGlyph> {
        &self.glyph_cache
    }
    
    /// 获取指定 glyph_id 的缓存条目（任意子像素桶）
    pub fn get_glyph_cache_entry(&self, glyph_id: GlyphId) -> Option<&RasterizedGlyph> {
        self.glyph_cache.get(&GlyphCacheKey { glyph_id, subpixel_bin: 0 })
    }
    
    pub fn font(&self) -> &FontVec {
        &self.fonts[0]
    }
    
    pub fn font_count(&self) -> usize {
        self.fonts.len()
    }
}

/// 计算子像素 coverage
/// subpixel_center: 子像素的中心 x 坐标（相对于像素）
/// glyph_coverage: 字形在该像素的覆盖率
fn compute_subpixel_coverage(subpixel_center: f32, glyph_coverage: f32) -> f32 {
    // 简单的子像素覆盖率计算
    // 假设子像素宽度为 1/3 像素
    let subpixel_width = 1.0 / 3.0;
    let half_width = subpixel_width / 2.0;
    
    // 子像素覆盖 = glyph_coverage * (子像素在 glyph 覆盖区域内的比例)
    // 这里简化为：如果子像素中心在 glyph 覆盖区域内，返回 glyph_coverage
    // 否则返回 0
    if subpixel_center >= -half_width && subpixel_center <= 1.0 + half_width {
        glyph_coverage
    } else {
        0.0
    }
}

/// 应用 gamma 校正
fn apply_gamma(coverage: f32, gamma: f32) -> f32 {
    if (gamma - 1.0).abs() < 0.01 {
        return coverage;
    }
    coverage.powf(1.0 / gamma)
}

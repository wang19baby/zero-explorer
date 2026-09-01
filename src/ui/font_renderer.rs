use ab_glyph::{FontVec, PxScale, GlyphId, Font, ScaleFont};
use std::collections::HashMap;

/// 预光栅化的字形数据
#[derive(Clone, Debug)]
pub struct RasterizedGlyph {
    pub width: u32,
    pub height: u32,
    pub bearing_x: f32,
    pub bearing_y: f32,
    pub advance: f32,
    pub pixels: Vec<u8>,
}

/// 字体渲染器 - 负责字形光栅化和缓存
pub struct FontRenderer {
    font: FontVec,
    font_size: f32,
    glyph_cache: HashMap<GlyphId, RasterizedGlyph>,
}

impl FontRenderer {
    pub fn new(font_data: Vec<u8>, font_size: f32) -> Result<Self, String> {
        let font = FontVec::try_from_vec(font_data)
            .map_err(|e| format!("Failed to load font: {}", e))?;
        
        let mut renderer = Self {
            font,
            font_size,
            glyph_cache: HashMap::new(),
        };
        
        renderer.pre_rasterize_ascii();
        
        Ok(renderer)
    }
    
    fn pre_rasterize_ascii(&mut self) {
        for c in 32u8..=126 {
            if let Some(ch) = char::from_u32(c as u32) {
                self.rasterize_glyph(ch);
            }
        }
    }
    
    pub fn rasterize_glyph(&mut self, ch: char) -> Option<&RasterizedGlyph> {
        let glyph_id = Font::glyph_id(&self.font, ch);
        
        if self.glyph_cache.contains_key(&glyph_id) {
            return self.glyph_cache.get(&glyph_id);
        }
        
        let glyph = glyph_id.with_scale_and_position(
            PxScale::from(self.font_size),
            ab_glyph::Point { x: 0.0, y: 0.0 },
        );
        
        let outlined = self.font.outline_glyph(glyph)?;
        let bounds = outlined.px_bounds();
        
        let width = (bounds.width() as u32).max(1);
        let height = (bounds.height() as u32).max(1);
        
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        
        outlined.draw(|x, y, coverage| {
            let px = x as i32 + bounds.min.x as i32;
            let py = y as i32 + bounds.min.y as i32;
            
            if px >= 0 && py >= 0 && (px as u32) < width && (py as u32) < height {
                let idx = ((py as u32 * width + px as u32) * 4) as usize;
                if idx + 3 < pixels.len() {
                    pixels[idx] = 255;
                    pixels[idx + 1] = 255;
                    pixels[idx + 2] = 255;
                    pixels[idx + 3] = (coverage * 255.0) as u8;
                }
            }
        });
        
        let scale_font = self.font.as_scaled(PxScale::from(self.font_size));
        let rasterized = RasterizedGlyph {
            width,
            height,
            bearing_x: bounds.min.x,
            bearing_y: bounds.min.y,
            advance: ScaleFont::h_advance(&scale_font, glyph_id),
            pixels,
        };
        
        self.glyph_cache.insert(glyph_id, rasterized);
        self.glyph_cache.get(&glyph_id)
    }
    
    pub fn get_glyph(&mut self, ch: char) -> Option<&RasterizedGlyph> {
        let glyph_id = Font::glyph_id(&self.font, ch);
        
        if !self.glyph_cache.contains_key(&glyph_id) {
            self.rasterize_glyph(ch);
        }
        
        self.glyph_cache.get(&glyph_id)
    }
    
    pub fn glyph_id(&self, ch: char) -> GlyphId {
        Font::glyph_id(&self.font, ch)
    }
    
    pub fn measure_text(&self, text: &str) -> f32 {
        let scale_font = self.font.as_scaled(PxScale::from(self.font_size));
        let mut width = 0.0;
        for ch in text.chars() {
            let glyph_id = Font::glyph_id(&self.font, ch);
            if let Some(glyph) = self.glyph_cache.get(&glyph_id) {
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
    
    pub fn glyph_cache(&self) -> &HashMap<GlyphId, RasterizedGlyph> {
        &self.glyph_cache
    }
    
    pub fn font(&self) -> &FontVec {
        &self.font
    }
}

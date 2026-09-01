#[derive(Debug, Clone)]
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
}

#[derive(Debug, Clone)]
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
            background: Color::from_hex(0xF5F5F5),
            surface: Color::from_hex(0xFFFFFF),
            surface_variant: Color::from_hex(0xF0F0F0),
            text_primary: Color::from_hex(0x1A1A1A),
            text_secondary: Color::from_hex(0x606060),
            text_disabled: Color::from_hex(0xB0B0B0),
            primary: Color::from_hex(0x2196F3),
            primary_light: Color::from_hex(0x64B5F6),
            primary_dark: Color::from_hex(0x1976D2),
            accent: Color::from_hex(0x9C27B0),
            accent_light: Color::from_hex(0xBA68C8),
            hover: Color::from_hex(0xE0E0E0),
            pressed: Color::from_hex(0xD0D0D0),
            focused: Color::from_hex(0x2196F3),
            selected: Color::from_hex(0x2196F3),
            border: Color::from_hex(0xE0E0E0),
            border_light: Color::from_hex(0xD0D0D0),
            success: Color::from_hex(0x4CAF50),
            warning: Color::from_hex(0xFF9800),
            error: Color::from_hex(0xF44336),
            info: Color::from_hex(0x2196F3),
            folder: Color::from_hex(0xFFC107),
            file: Color::from_hex(0x9E9E9E),
            image: Color::from_hex(0xF44336),
            video: Color::from_hex(0x9C27B0),
            audio: Color::from_hex(0x2196F3),
            document: Color::from_hex(0x4CAF50),
            archive: Color::from_hex(0xFF9800),
            code: Color::from_hex(0x00BCD4),
        }
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
}

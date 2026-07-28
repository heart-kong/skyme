/// A colour in premultiplied alpha format (matching D2D1_COLOR_F).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self { Self { r, g, b, a } }

    pub const fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0, a: a as f32 / 255.0 }
    }

    /// Parse a hex colour string (`#RGB`, `#RRGGBB`, or `#AARRGGBB`).
    pub fn parse(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        let (r, g, b, a) = match hex.len() {
            3 => (
                u8::from_str_radix(&hex[0..1], 16).ok()? * 17,
                u8::from_str_radix(&hex[1..2], 16).ok()? * 17,
                u8::from_str_radix(&hex[2..3], 16).ok()? * 17,
                255,
            ),
            6 => (
                u8::from_str_radix(&hex[0..2], 16).ok()?,
                u8::from_str_radix(&hex[2..4], 16).ok()?,
                u8::from_str_radix(&hex[4..6], 16).ok()?,
                255,
            ),
            8 => (
                u8::from_str_radix(&hex[2..4], 16).ok()?,
                u8::from_str_radix(&hex[4..6], 16).ok()?,
                u8::from_str_radix(&hex[6..8], 16).ok()?,
                u8::from_str_radix(&hex[0..2], 16).ok()?,
            ),
            _ => return None,
        };
        Some(Self::from_rgba8(r, g, b, a))
    }

    pub const BLACK: Color = Color::new(0.0, 0.0, 0.0, 1.0);
    pub const WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);
    pub const TRANSPARENT: Color = Color::new(0.0, 0.0, 0.0, 0.0);
}

impl Default for Color {
    fn default() -> Self { Self::BLACK }
}

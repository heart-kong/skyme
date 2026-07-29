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

#[cfg(test)]
mod tests {
    use crate::color::Color;

    #[test]
    fn test_black() {
        assert_eq!(Color::BLACK, Color::new(0.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn test_white() {
        assert_eq!(Color::WHITE, Color::new(1.0, 1.0, 1.0, 1.0));
    }

    #[test]
    fn test_transparent() {
        assert_eq!(Color::TRANSPARENT, Color::new(0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn test_from_rgba8() {
        let c = Color::from_rgba8(255, 128, 64, 255);
        assert!((c.r - 1.0).abs() < 0.001);
        assert!((c.g - 0.502).abs() < 0.001);
        assert!((c.b - 0.251).abs() < 0.001);
        assert!((c.a - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_parse_hex_6() {
        let c = Color::parse("#FF8000").unwrap();
        assert!((c.r - 1.0).abs() < 0.001);
        assert!((c.g - 0.502).abs() < 0.001);
        assert_eq!(c.b, 0.0);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn test_parse_hex_3() {
        let c = Color::parse("#F80").unwrap();
        assert!((c.r - 1.0).abs() < 0.001);
        assert!((c.g - 0.533).abs() < 0.01);
        assert_eq!(c.b, 0.0);
    }

    #[test]
    fn test_parse_hex_8() {
        let c = Color::parse("#80FF8000").unwrap();
        assert!((c.a - 0.502).abs() < 0.001);
        assert!((c.r - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_parse_invalid() {
        assert!(Color::parse("notacolor").is_none());
        assert!(Color::parse("").is_none());
        assert!(Color::parse("#GGG").is_none());
    }

    #[test]
    fn test_default_is_black() {
        assert_eq!(Color::default(), Color::BLACK);
    }

    #[test]
    fn test_partial_eq() {
        let a = Color::new(0.5, 0.5, 0.5, 1.0);
        let b = Color::new(0.5, 0.5, 0.5, 1.0);
        assert_eq!(a, b);
    }
}

//! DirectWrite text layout and font management.
//!
//! Provides text measurement, layout, and rendering support.

use std::collections::HashMap;

/// Font description.
pub struct Font {
    pub family_name: String,
    pub size: f32,
    pub weight: FontWeight,
    pub style: FontStyle,
}

impl Font {
    pub fn new(family_name: &str, size: f32) -> Self {
        Self { family_name: family_name.into(), size, weight: FontWeight::Normal, style: FontStyle::Normal }
    }
    pub fn with_weight(mut self, weight: FontWeight) -> Self { self.weight = weight; self }
    pub fn with_style(mut self, style: FontStyle) -> Self { self.style = style; self }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum FontWeight { #[default] Normal, Bold, SemiBold, Light, Medium }

#[derive(Clone, Copy, Debug, Default)]
pub enum FontStyle { #[default] Normal, Italic, Oblique }

/// Laid-out text ready for rendering.
pub struct TextLayout {
    pub text: String,
    pub width: f32,
    pub height: f32,
    pub font: Font,
}

impl TextLayout {
    pub fn new(text: &str, font: Font) -> Self {
        let (w, h) = measure_text_approx(text, &font);
        Self { text: text.to_owned(), width: w, height: h, font }
    }
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_owned();
        let (w, h) = measure_text_approx(text, &self.font);
        self.width = w; self.height = h;
    }
}

/// Approximate text measurement (placeholder — real impl uses DWrite).
fn measure_text_approx(text: &str, font: &Font) -> (f32, f32) {
    (text.chars().count() as f32 * font.size * 0.55, font.size * 1.35)
}

/// Manages font resources.
pub struct DWriteFactory {
    fonts: HashMap<String, Font>,
}

impl DWriteFactory {
    pub fn new() -> Self { Self { fonts: HashMap::new() } }
    pub fn register_font(&mut self, font: Font) { self.fonts.insert(font.family_name.clone(), font); }
    pub fn get_font(&self, name: &str) -> Option<&Font> { self.fonts.get(name) }
}

impl Default for DWriteFactory { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use crate::dwrite::*;

    #[test]
    fn test_font_new() {
        let f = Font::new("Segoe UI", 14.0);
        assert_eq!(f.family_name, "Segoe UI");
        assert_eq!(f.size, 14.0);
    }

    #[test]
    fn test_font_with_weight() {
        let f = Font::new("Arial", 12.0).with_weight(FontWeight::Bold);
        assert!(matches!(f.weight, FontWeight::Bold));
    }

    #[test]
    fn test_text_layout() {
        let font = Font::new("Test", 10.0);
        let layout = TextLayout::new("Hello", font);
        assert_eq!(layout.text, "Hello");
        assert!(layout.width > 0.0);
        assert!(layout.height > 0.0);
    }

    #[test]
    fn test_text_layout_set_text() {
        let font = Font::new("Test", 10.0);
        let mut layout = TextLayout::new("Hello", font);
        layout.set_text("World");
        assert_eq!(layout.text, "World");
    }

    #[test]
    fn test_dwrite_factory() {
        let mut f = DWriteFactory::new();
        f.register_font(Font::new("Custom", 16.0));
        assert!(f.get_font("Custom").is_some());
        assert!(f.get_font("Nonexistent").is_none());
    }

    #[test]
    fn test_font_weight_default() {
        assert!(matches!(FontWeight::default(), FontWeight::Normal));
    }

    #[test]
    fn test_font_style_default() {
        assert!(matches!(FontStyle::default(), FontStyle::Normal));
    }
}

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

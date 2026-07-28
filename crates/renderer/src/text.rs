//! Text rendering utilities bridging DWrite layout to D2D drawing.

use crate::dwrite::{Font, TextLayout};
use crate::color::Color;
use skyme_common::Rect;

/// A fully laid-out text element.
pub struct TextElement {
    pub layout: TextLayout,
    pub color: Color,
    pub background_color: Option<Color>,
    pub position: Rect,
}

impl TextElement {
    pub fn new(text: &str, font: Font, color: Color) -> Self {
        Self { layout: TextLayout::new(text, font), color, background_color: None, position: Rect::default() }
    }
    pub fn with_position(mut self, x: f32, y: f32, w: f32, h: f32) -> Self {
        self.position = Rect { x, y, width: w, height: h }; self
    }
    pub fn with_background(mut self, color: Color) -> Self { self.background_color = Some(color); self }
}

/// A styled text run with multiple segments.
pub struct StyledText { pub segments: Vec<StyledSegment> }

/// A single segment of styled text (for preedit highlighting).
pub struct StyledSegment {
    pub text: String,
    pub font: Font,
    pub color: Color,
    pub underline: bool,
}

//! GPU-accelerated rendering layer for Skyme.
//!
//! Provides a cross-platform abstraction over:
//! - **Direct2D** — 2D vector rendering (Windows)
//! - **DirectWrite** — text layout & typography (Windows)
//! - **DirectComposition** — visual tree & composition (Windows)
//!
//! All platform-specific code is gated behind `cfg(target_os = "windows")`.
//! On other platforms a simple pixel-buffer renderer is used for development.

pub mod color;
pub mod text;
pub mod d2d;
pub mod dwrite;
pub mod composition;

pub use color::Color;
pub use text::{TextElement, StyledText, StyledSegment};
pub use d2d::D2DRenderer;
pub use dwrite::{Font, FontWeight, FontStyle, TextLayout, DWriteFactory};
pub use composition::{VisualNode, CompositionTree};

use skyme_common::Rect;

/// Top-level rendering abstraction implemented by all backends.
///
/// # Implementations
///
/// | Backend | Platform | Feature |
/// |---------|----------|---------|
/// | `D2DRenderer` | Windows | Direct2D + DirectWrite + DirectComposition |
/// | `NullRenderer` | Any | No-op (for testing/development) |
pub trait Renderer {
    /// Begin a new frame. Returns false if the render target is unavailable.
    fn begin_frame(&mut self) -> bool;

    /// End the current frame and present to the screen.
    fn end_frame(&mut self);

    /// Fill a rectangle with a solid colour.
    fn fill_rect(&mut self, rect: &Rect, color: &Color);

    /// Draw a rectangle outline.
    fn stroke_rect(&mut self, rect: &Rect, color: &Color, stroke_width: f32);

    /// Draw text at the given position.
    fn draw_text(&mut self, text: &str, font: &Font, color: &Color, x: f32, y: f32);

    /// Draw text within a bounding rect (word-wrapped if needed).
    fn draw_text_in_rect(&mut self, text: &str, font: &Font, color: &Color, rect: &Rect);

    /// Measure the pixel dimensions of a string.
    fn measure_text(&self, text: &str, font: &Font) -> (f32, f32);

    /// Push a rounded-rect clip region.
    fn push_clip_rounded_rect(&mut self, rect: &Rect, radius: f32);

    /// Pop the last clip region.
    fn pop_clip(&mut self);

    /// Get the current surface size in DIP.
    fn size(&self) -> (f32, f32);

    /// Get the DPI scaling factor.
    fn dpi_scale(&self) -> f32;
}

/// Null renderer — discards all drawing commands.
/// Useful for testing and development on non-Windows platforms.
pub struct NullRenderer;

impl NullRenderer {
    pub fn new() -> Self { Self }
}

impl Renderer for NullRenderer {
    fn begin_frame(&mut self) -> bool { true }
    fn end_frame(&mut self) {}
    fn fill_rect(&mut self, _rect: &Rect, _color: &Color) {}
    fn stroke_rect(&mut self, _rect: &Rect, _color: &Color, _w: f32) {}
    fn draw_text(&mut self, _text: &str, _font: &Font, _color: &Color, _x: f32, _y: f32) {}
    fn draw_text_in_rect(&mut self, _text: &str, _font: &Font, _color: &Color, _rect: &Rect) {}
    fn measure_text(&self, text: &str, font: &Font) -> (f32, f32) {
        (text.len() as f32 * font.size * 0.5, font.size * 1.3)
    }
    fn push_clip_rounded_rect(&mut self, _rect: &Rect, _radius: f32) {}
    fn pop_clip(&mut self) {}
    fn size(&self) -> (f32, f32) { (0.0, 0.0) }
    fn dpi_scale(&self) -> f32 { 1.0 }
}

impl Default for NullRenderer { fn default() -> Self { Self::new() } }

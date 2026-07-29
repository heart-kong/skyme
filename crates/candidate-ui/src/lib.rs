//! Candidate window and inline-candidate rendering logic.
//!
//! Renders candidate lists using different display strategies:
//! - **Floating** — popup window near cursor (traditional Weasel style)
//! - **Inline** — candidates rendered inside preedit (macOS Yume style)
//! - **Classic** — vertical list with select-key labels
//! - **Dock** — docked at screen edge
//!
//! Each mode implements [`CandidateRenderer`]. Themes control colours
//! and fonts but do not change rendering logic.

pub mod floating;
pub mod inline;
pub mod classic;
pub mod dock;

pub use floating::FloatingRenderer;
pub use inline::YumeInlinePresenter;
pub use classic::ClassicRenderer;
pub use dock::DockRenderer;

use skyme_common::{Candidate, DisplayMode, Rect};
use skyme_renderer::Renderer;

/// Core trait for rendering a candidate list.
pub trait CandidateRenderer {
    /// Render candidates using the provided renderer and layout info.
    fn render(&mut self, renderer: &mut dyn Renderer, candidates: &[Candidate], display_mode: DisplayMode, viewport: &Rect);

    /// Measure how much space the candidate window needs.
    fn measure(&self, candidates: &[Candidate], display_mode: DisplayMode) -> Rect;
}

/// Inline candidate presenter (macOS Yume style).
///
/// Renders candidates inline within the composition/preedit text
/// rather than in a separate window.
pub trait InlinePresenter {
    fn begin(&mut self, preedit: &str, cursor_pos: usize);
    fn update(&mut self, preedit: &str, candidates: &[Candidate], cursor_pos: usize);
    fn commit(&mut self) -> String;
    fn cancel(&mut self);
}

/// Shared rendering constants used by all display modes.
pub(crate) mod layout {
    use skyme_renderer::Color;
    use skyme_renderer::Font;

    pub const PADDING: f32 = 8.0;
    pub const CANDIDATE_SPACING: f32 = 4.0;
    pub const LABEL_SPACING: f32 = 6.0;
    pub const CORNER_RADIUS: f32 = 8.0;
    pub const BORDER_WIDTH: f32 = 1.0;

    pub fn default_font() -> Font { Font::new("Segoe UI", 14.0) }
    pub fn label_font() -> Font { Font::new("Segoe UI", 12.0) }
    pub fn comment_font() -> Font { Font::new("Segoe UI", 12.0) }

    pub fn bg_color() -> Color { Color::from_rgba8(32, 33, 36, 240) }
    pub fn text_color() -> Color { Color::from_rgba8(232, 234, 237, 255) }
    #[allow(dead_code)]
    pub fn highlight_bg() -> Color { Color::from_rgba8(138, 180, 248, 60) }
    #[allow(dead_code)]
    pub fn highlight_text() -> Color { Color::from_rgba8(138, 180, 248, 255) }
    pub fn comment_color() -> Color { Color::from_rgba8(154, 160, 166, 255) }
    pub fn border_color() -> Color { Color::from_rgba8(95, 99, 104, 255) }
    pub fn label_color() -> Color { Color::from_rgba8(138, 180, 248, 200) }
    pub fn hover_bg() -> Color { Color::from_rgba8(60, 64, 67, 255) }
    pub fn selected_bg() -> Color { Color::from_rgba8(138, 180, 248, 30) }
}

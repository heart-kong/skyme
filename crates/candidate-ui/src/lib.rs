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

#[cfg(test)]
mod tests {
    use crate::layout;
    use crate::*;
    use skyme_common::{Candidate, DisplayMode, Rect};
    use skyme_renderer::NullRenderer;

    #[test]
    fn test_layout_constants() {
        assert!(layout::PADDING > 0.0);
        assert!(layout::CORNER_RADIUS > 0.0);
    }

    #[test]
    fn test_floating_measure_empty() {
        let r = FloatingRenderer::new();
        let rect = r.measure(&[], DisplayMode::Floating);
        assert_eq!(rect.width, 0.0);
        assert_eq!(rect.height, 0.0);
    }

    #[test]
    fn test_floating_measure_with_candidates() {
        let r = FloatingRenderer::new();
        let candidates = vec![Candidate { text: "测试".into(), comment: "test".into(), index: 0, quality: 1.0 }];
        let rect = r.measure(&candidates, DisplayMode::Floating);
        assert!(rect.width > 0.0);
        assert!(rect.height > 0.0);
    }

    #[test]
    fn test_floating_render_no_crash() {
        let mut r = FloatingRenderer::new();
        let mut renderer = NullRenderer::new();
        r.render(&mut renderer, &[], DisplayMode::Floating, &Rect::default());
    }

    #[test]
    fn test_floating_render_with_candidates() {
        let mut r = FloatingRenderer::new();
        let mut renderer = NullRenderer::new();
        let candidates = vec![
            Candidate { text: "中".into(), comment: "zhong".into(), index: 0, quality: 0.9 },
            Candidate { text: "国".into(), comment: "guo".into(), index: 1, quality: 0.8 },
        ];
        r.set_selected(0);
        r.render(&mut renderer, &candidates, DisplayMode::Floating, &Rect { x: 0.0, y: 0.0, width: 800.0, height: 600.0 });
    }

    #[test]
    fn test_classic_measure() {
        let r = ClassicRenderer::new();
        let candidates = vec![Candidate { text: "hello".into(), comment: "".into(), index: 0, quality: 1.0 }];
        let rect = r.measure(&candidates, DisplayMode::Classic);
        assert!(rect.width > 0.0);
    }

    #[test]
    fn test_classic_render() {
        let mut r = ClassicRenderer::new();
        let mut renderer = NullRenderer::new();
        let candidates = vec![
            Candidate { text: "你好".into(), comment: "hello".into(), index: 0, quality: 1.0 },
        ];
        r.render(&mut renderer, &candidates, DisplayMode::Classic, &Rect::default());
    }

    #[test]
    fn test_dock_measure() {
        let r = DockRenderer::new();
        let candidates = vec![Candidate { text: "test".into(), comment: "".into(), index: 0, quality: 1.0 }];
        let rect = r.measure(&candidates, DisplayMode::Dock);
        assert!(rect.height == 48.0);
    }

    #[test]
    fn test_dock_render() {
        let mut r = DockRenderer::new();
        let mut renderer = NullRenderer::new();
        let candidates = vec![
            Candidate { text: "中".into(), comment: "".into(), index: 0, quality: 0.9 },
            Candidate { text: "国".into(), comment: "".into(), index: 1, quality: 0.8 },
        ];
        let viewport = Rect { x: 0.0, y: 0.0, width: 1920.0, height: 1080.0 };
        r.render(&mut renderer, &candidates, DisplayMode::Dock, &viewport);
    }

    #[test]
    fn test_inline_presenter() {
        let mut p = YumeInlinePresenter::new();
        p.begin("ni", 2);
        p.update("nihao", &[Candidate { text: "你好".into(), comment: "".into(), index: 0, quality: 1.0 }], 5);
        let text = p.commit();
        assert!(text.contains("nihao"));
        assert!(text.contains("你好") || text.contains('①'));
    }

    #[test]
    fn test_inline_presenter_cancel() {
        let mut p = YumeInlinePresenter::new();
        p.begin("test", 4);
        p.cancel();
        let text = p.commit();
        assert_eq!(text, "");
    }

    #[test]
    fn test_inline_empty_candidates() {
        let mut p = YumeInlinePresenter::new();
        p.begin("hello", 5);
        p.update("hello", &[], 5);
        let text = p.commit();
        assert_eq!(text, "hello");
    }

    #[test]
    fn test_display_mode_enum() {
        assert_ne!(DisplayMode::Floating as u8, DisplayMode::Inline as u8);
        assert_ne!(DisplayMode::Dock as u8, DisplayMode::Classic as u8);
    }
}

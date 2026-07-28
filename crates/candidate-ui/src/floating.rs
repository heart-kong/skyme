//! Floating candidate window renderer.
//!
//! Renders a popup window near the text insertion point,
//! similar to the traditional Weasel candidate window.

use crate::layout;
use crate::CandidateRenderer;
use skyme_common::{Candidate, DisplayMode, Rect};
use skyme_renderer::Renderer;

/// Floating candidate window renderer.
pub struct FloatingRenderer {
    selected_index: usize,
    hovered_index: Option<usize>,
}

impl FloatingRenderer {
    pub fn new() -> Self {
        Self { selected_index: 0, hovered_index: None }
    }

    pub fn set_selected(&mut self, index: usize) { self.selected_index = index; }
    pub fn set_hovered(&mut self, index: Option<usize>) { self.hovered_index = index; }
}

impl Default for FloatingRenderer { fn default() -> Self { Self::new() } }

impl CandidateRenderer for FloatingRenderer {
    fn render(&mut self, r: &mut dyn Renderer, candidates: &[Candidate], _mode: DisplayMode, _viewport: &Rect) {
        if candidates.is_empty() { return; }

        let rect = self.measure(candidates, DisplayMode::Floating);
        let font = layout::default_font();
        let label_font = layout::label_font();
        let comment_f = layout::comment_font();

        // Background
        r.push_clip_rounded_rect(&rect, layout::CORNER_RADIUS);
        r.fill_rect(&rect, &layout::bg_color());
        r.stroke_rect(&rect, &layout::border_color(), layout::BORDER_WIDTH);

        let mut y = rect.y + layout::PADDING;
        for (i, cand) in candidates.iter().enumerate() {
            let x = rect.x + layout::PADDING;
            let label = format!("{}", i + 1);
            let (label_w, _) = r.measure_text(&label, &label_font);
            let (text_w, text_h) = r.measure_text(&cand.text, &font);
            let (_comm_w, _) = if !cand.comment.is_empty() {
                r.measure_text(&cand.comment, &comment_f)
            } else { (0.0, 0.0) };

            let item_h = text_h.max(20.0);
            let item_rect = Rect { x: rect.x + 1.0, y, width: rect.width - 2.0, height: item_h + layout::CANDIDATE_SPACING };

            // Highlight if selected
            if i == self.selected_index {
                r.fill_rect(&item_rect, &layout::selected_bg());
            } else if self.hovered_index == Some(i) {
                r.fill_rect(&item_rect, &layout::hover_bg());
            }

            // Label
            r.draw_text(&label, &label_font, &layout::label_color(), x, y + 2.0);

            // Text
            let tx = x + label_w + layout::LABEL_SPACING;
            r.draw_text(&cand.text, &font, &layout::text_color(), tx, y + 2.0);

            // Comment
            if !cand.comment.is_empty() {
                let cx = tx + text_w + layout::LABEL_SPACING;
                r.draw_text(&cand.comment, &comment_f, &layout::comment_color(), cx, y + 4.0);
            }

            y += item_h + layout::CANDIDATE_SPACING;
        }

        r.pop_clip();
    }

    fn measure(&self, candidates: &[Candidate], _mode: DisplayMode) -> Rect {
        if candidates.is_empty() { return Rect::default(); }
        let font = layout::default_font();

        let mut width: f32 = 200.0;
        let item_h = font.size * 1.35;
        let height = layout::PADDING * 2.0 + candidates.len() as f32 * (item_h + layout::CANDIDATE_SPACING) + 2.0;

        for cand in candidates {
            let text_w = cand.text.len() as f32 * font.size * 0.55;
            let comm_w = cand.comment.len() as f32 * font.size * 0.5;
            let total = text_w + comm_w + 40.0;
            width = width.max(total);
        }

        Rect { x: 0.0, y: 0.0, width: width + layout::PADDING * 2.0, height }
    }
}

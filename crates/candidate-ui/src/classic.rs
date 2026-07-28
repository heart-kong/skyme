//! Classic Weasel-style candidate window renderer.
//!
//! Renders candidates in a vertical list with select-key labels.

use crate::layout;
use crate::CandidateRenderer;
use skyme_common::{Candidate, DisplayMode, Rect};
use skyme_renderer::Renderer;

/// Classic vertical-list candidate renderer.
pub struct ClassicRenderer {
    selected_index: usize,
}

impl ClassicRenderer {
    pub fn new() -> Self { Self { selected_index: 0 } }
    pub fn set_selected(&mut self, index: usize) { self.selected_index = index; }
}

impl Default for ClassicRenderer { fn default() -> Self { Self::new() } }

impl CandidateRenderer for ClassicRenderer {
    fn render(&mut self, r: &mut dyn Renderer, candidates: &[Candidate], _mode: DisplayMode, _viewport: &Rect) {
        if candidates.is_empty() { return; }

        let rect = self.measure(candidates, DisplayMode::Classic);
        let font = layout::default_font();
        let comment_f = layout::comment_font();

        // Background + border
        r.push_clip_rounded_rect(&rect, layout::CORNER_RADIUS);
        r.fill_rect(&rect, &layout::bg_color());
        r.stroke_rect(&rect, &layout::border_color(), layout::BORDER_WIDTH);

        let mut y = rect.y + layout::PADDING;
        for (i, cand) in candidates.iter().enumerate() {
            let x = rect.x + layout::PADDING;
            let select_key = format!("{}", i + 1);

            let item_h = font.size * 1.35;
            let item_rect = Rect { x: rect.x + 1.0, y, width: rect.width - 2.0, height: item_h + layout::CANDIDATE_SPACING };

            // Selection highlight
            if i == self.selected_index {
                r.fill_rect(&item_rect, &layout::selected_bg());
            }

            // Select key + candidate text on one line
            let label = format!("{}. {}", select_key, cand.text);
            r.draw_text(&label, &font, &layout::text_color(), x, y + 2.0);

            // Comment right-aligned
            if !cand.comment.is_empty() {
                let (comm_w, _) = r.measure_text(&cand.comment, &comment_f);
                let cx = rect.x + rect.width - layout::PADDING - comm_w;
                r.draw_text(&cand.comment, &comment_f, &layout::comment_color(), cx, y + 4.0);
            }

            y += item_h + layout::CANDIDATE_SPACING;
        }

        r.pop_clip();
    }

    fn measure(&self, candidates: &[Candidate], _mode: DisplayMode) -> Rect {
        if candidates.is_empty() { return Rect::default(); }
        let font = layout::default_font();

        let mut width: f32 = 240.0;
        for cand in candidates {
            let text_w = cand.text.len() as f32 * font.size * 0.55;
            let comm_w = cand.comment.len() as f32 * font.size * 0.5;
            width = width.max(text_w + comm_w + 60.0);
        }

        let item_h = font.size * 1.35;
        let height = layout::PADDING * 2.0 + candidates.len() as f32 * (item_h + layout::CANDIDATE_SPACING);
        Rect { x: 0.0, y: 0.0, width: width + layout::PADDING * 2.0, height }
    }
}

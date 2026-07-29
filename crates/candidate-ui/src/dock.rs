//! Dock candidate bar — rendered at the bottom of the screen.
//!
//! Similar to Windows 11 touch keyboard candidate bar: a horizontal
//! strip spanning most of the screen width, positioned at the bottom.

use crate::layout;
use crate::CandidateRenderer;
use skyme_common::{Candidate, DisplayMode, Rect};
use skyme_renderer::Renderer;

/// Dock-style candidate bar renderer.
///
/// Candidates are arranged horizontally in a single row.
/// The bar spans the full viewport width at the bottom.
pub struct DockRenderer {
    selected_index: usize,
    /// Offset for horizontal scrolling (in candidate slots).
    scroll_offset: usize,
    /// Max candidates visible at once.
    visible_count: usize,
}

impl DockRenderer {
    pub fn new() -> Self {
        Self { selected_index: 0, scroll_offset: 0, visible_count: 10 }
    }

    pub fn set_selected(&mut self, index: usize) { self.selected_index = index; }
    pub fn scroll_left(&mut self) { if self.scroll_offset > 0 { self.scroll_offset -= 1; } }
    pub fn scroll_right(&mut self, max: usize) {
        if self.scroll_offset + self.visible_count < max { self.scroll_offset += 1; }
    }
}

impl Default for DockRenderer { fn default() -> Self { Self::new() } }

impl CandidateRenderer for DockRenderer {
    fn render(&mut self, r: &mut dyn Renderer, candidates: &[Candidate], _mode: DisplayMode, viewport: &Rect) {
        if candidates.is_empty() { return; }

        let font = layout::default_font();
        let label_font = layout::label_font();
        let bar_height: f32 = 48.0;
        let bar_y = viewport.height - bar_height;

        let bar_rect = Rect {
            x: 0.0,
            y: bar_y,
            width: viewport.width,
            height: bar_height,
        };

        // Background — full width, no border radius on bottom (flush with screen edge)
        r.fill_rect(&bar_rect, &layout::bg_color());
        // Top border line
        r.stroke_rect(&Rect { x: 0.0, y: bar_y, width: viewport.width, height: 1.0 }, &layout::border_color(), 1.0);

        // Calculate candidate slot width
        let slot_count = self.visible_count.min(candidates.len());
        if slot_count == 0 { return; }

        let slot_width = viewport.width / slot_count as f32;
        let item_h = bar_height - 12.0;

        // Render visible candidates
        let start = self.scroll_offset;
        let end = (start + self.visible_count).min(candidates.len());

        for i in start..end {
            let idx = i - start;
            let cand = &candidates[i];

            let x = idx as f32 * slot_width;
            let item_rect = Rect { x, y: bar_y + 6.0, width: slot_width, height: item_h };

            // Selection highlight
            if i == self.selected_index {
                let hl_rect = Rect {
                    x: x + 4.0,
                    y: bar_y + 6.0,
                    width: slot_width - 8.0,
                    height: item_h,
                };
                r.fill_rect(&hl_rect, &layout::selected_bg());
            }

            // Select label
            let label = format!("{}", (i - start) + 1);
            let (label_w, _) = r.measure_text(&label, &label_font);

            // Candidate text (truncated if too long)
            let max_text_w = slot_width - label_w - layout::LABEL_SPACING - 16.0;
            let display_text = if cand.text.len() as f32 * font.size * 0.55 > max_text_w {
                let max_chars = (max_text_w / (font.size * 0.55)).floor() as usize;
                if max_chars > 2 {
                    format!("{}…", &cand.text[..max_chars.max(1)])
                } else {
                    cand.text.clone()
                }
            } else {
                cand.text.clone()
            };

            let tx = x + 8.0;
            r.draw_text(&label, &label_font, &layout::label_color(), tx, bar_y + 14.0);
            r.draw_text(&display_text, &font, &layout::text_color(), tx + label_w + layout::LABEL_SPACING, bar_y + 12.0);

            // Separator line (except last visible slot)
            if idx < end - start - 1 {
                let sep_x = x + slot_width - 1.0;
                r.fill_rect(&Rect { x: sep_x, y: bar_y + 10.0, width: 1.0, height: bar_height - 20.0 }, &layout::comment_color());
            }
        }

        // Scroll indicators
        if self.scroll_offset > 0 {
            // Left arrow
            r.draw_text("◀", &label_font, &layout::label_color(), 4.0, bar_y + 14.0);
        }
        if end < candidates.len() {
            // Right arrow
            r.draw_text("▶", &label_font, &layout::label_color(), viewport.width - 18.0, bar_y + 14.0);
        }
    }

    fn measure(&self, candidates: &[Candidate], _mode: DisplayMode) -> Rect {
        if candidates.is_empty() { return Rect::default(); }
        // Dock bar typically spans the full screen width.
        // Height is fixed. Width is set by the viewport.
        Rect { x: 0.0, y: 0.0, width: 1920.0, height: 48.0 }
    }
}

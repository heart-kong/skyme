//! macOS Yume-style inline candidate presenter.
//!
//! Candidates are displayed inline within the composition text
//! rather than in a separate floating window.

use crate::InlinePresenter;
use skyme_common::Candidate;

/// Yume-style inline candidate presenter.
///
/// When active, the preedit text is modified to show candidates
/// in-line (e.g. "zhong1①中②重③种").
pub struct YumeInlinePresenter {
    preedit: String,
    candidates: Vec<Candidate>,
    cursor_pos: usize,
}

impl YumeInlinePresenter {
    pub fn new() -> Self {
        Self { preedit: String::new(), candidates: Vec::new(), cursor_pos: 0 }
    }

    /// Build the inline display string from preedit + candidates.
    fn build_inline_text(&self) -> String {
        if self.candidates.is_empty() {
            return self.preedit.clone();
        }

        let mut result = self.preedit.clone();
        result.push(' ');
        for (i, cand) in self.candidates.iter().enumerate().take(9) {
            let marker = inline_marker(i);
            result.push_str(&format!("{}{} ", marker, cand.text));
        }
        result
    }
}

impl Default for YumeInlinePresenter { fn default() -> Self { Self::new() } }

impl InlinePresenter for YumeInlinePresenter {
    fn begin(&mut self, preedit: &str, cursor_pos: usize) {
        self.preedit = preedit.to_owned();
        self.cursor_pos = cursor_pos;
        self.candidates.clear();
    }

    fn update(&mut self, preedit: &str, candidates: &[Candidate], cursor_pos: usize) {
        self.preedit = preedit.to_owned();
        self.candidates = candidates.to_vec();
        self.cursor_pos = cursor_pos;
    }

    fn commit(&mut self) -> String {
        let text = self.build_inline_text();
        self.preedit.clear();
        self.candidates.clear();
        text
    }

    fn cancel(&mut self) {
        self.preedit.clear();
        self.candidates.clear();
        self.cursor_pos = 0;
    }
}

/// Create a small inline marker symbol (①, ②, ③, etc.).
fn inline_marker(i: usize) -> &'static str {
    match i {
        0 => "①", 1 => "②", 2 => "③", 3 => "④", 4 => "⑤",
        5 => "⑥", 6 => "⑦", 7 => "⑧", 8 => "⑨",
        _ => "•",
    }
}

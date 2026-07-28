//! TSF document / text store management.

/// Manages the TSF document (text store) for an input context.
///
/// Handles text buffer operations: insertion, deletion, selection, and
/// composition text display through TSF's ITfContext / ITfRange interfaces.
pub struct DocumentManager {
    text: String,
    selection_start: usize,
    selection_end: usize,
}

impl DocumentManager {
    pub fn new() -> Self { Self { text: String::new(), selection_start: 0, selection_end: 0 } }

    pub fn text(&self) -> &str { &self.text }
    pub fn set_text(&mut self, text: &str) { self.text = text.to_owned(); }
    pub fn selection(&self) -> (usize, usize) { (self.selection_start, self.selection_end) }
    pub fn set_selection(&mut self, start: usize, end: usize) { self.selection_start = start; self.selection_end = end; }
    pub fn clear(&mut self) { self.text.clear(); self.selection_start = 0; self.selection_end = 0; }
}

impl Default for DocumentManager { fn default() -> Self { Self::new() } }

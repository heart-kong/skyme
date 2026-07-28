/// A single candidate from the Rime engine.
#[derive(Clone, Debug, Default)]
pub struct Candidate {
    /// Display text (what the user sees).
    pub text: String,
    /// Comment / annotation (e.g. "| 中 | zhong1").
    pub comment: String,
    /// Candidate index inside the current page.
    pub index: u32,
    /// Matching quality / score (engine-specific).
    pub quality: f64,
}

/// Geometry used by the renderer for positioning UI elements.
#[derive(Clone, Copy, Debug, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Preedit / composition segment.
#[derive(Clone, Debug, Default)]
pub struct PreeditSegment {
    pub text: String,
    pub start: usize,
    pub end: usize,
    /// Highlighted (the cursor is on this segment / selected).
    pub highlighted: bool,
}

/// Composition state.
#[derive(Clone, Debug, Default)]
pub struct CompositionState {
    pub preedit: String,
    pub cursor_pos: usize,
    pub segments: Vec<PreeditSegment>,
    pub candidates: Vec<Candidate>,
    pub page: u32,
    pub total_pages: u32,
}

/// Supported UI display modes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DisplayMode {
    /// Candidates appear inline in the composition text (macOS Yume style).
    Inline,
    /// A floating window near the cursor.
    #[default]
    Floating,
    /// Docked at the bottom of the screen.
    Dock,
    /// Classic Weasel-style window.
    Classic,
}

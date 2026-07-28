//! TSF input context tracking.

use crate::document::DocumentManager;

/// Represents a TSF input context (a text field / edit control).
pub struct InputContext {
    pub id: u64,
    pub document: DocumentManager,
}

impl InputContext {
    pub fn new(id: u64) -> Self { Self { id, document: DocumentManager::new() } }

    /// Update the TSF composition window / display attributes.
    pub fn update(&mut self) {}

    /// Commit text through TSF into the application.
    pub fn commit_text(&mut self, _text: &str) {}

    /// Cancel the current composition in this context.
    pub fn cancel(&mut self) {}
}

/// Trait abstracting TSF input context operations.
pub trait InputContextOps {
    fn update(&mut self);
    fn commit(&mut self, text: &str);
    fn cancel(&mut self);
}

impl InputContextOps for InputContext {
    fn update(&mut self) { self.update(); }
    fn commit(&mut self, text: &str) { self.commit_text(text); }
    fn cancel(&mut self) { self.cancel(); }
}

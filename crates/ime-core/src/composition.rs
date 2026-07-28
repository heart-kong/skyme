//! Composition lifecycle management.

/// Manages the TSF composition lifecycle.
///
/// A composition begins when the user starts typing (first character)
/// and ends when text is committed or the composition is cancelled.
pub struct CompositionManager {
    is_composing: bool,
    session_id: u64,
}

impl CompositionManager {
    pub fn new() -> Self { Self { is_composing: false, session_id: 0 } }

    pub fn start(&mut self, session_id: u64) {
        self.is_composing = true;
        self.session_id = session_id;
        log::debug!("Composition started (session {})", session_id);
    }

    pub fn end(&mut self) {
        self.is_composing = false;
        log::debug!("Composition ended");
    }

    pub fn cancel(&mut self) {
        self.is_composing = false;
        log::debug!("Composition cancelled");
    }

    pub fn is_active(&self) -> bool { self.is_composing }
    pub fn session_id(&self) -> u64 { self.session_id }
}

impl Default for CompositionManager { fn default() -> Self { Self::new() } }

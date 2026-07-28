use skyme_common::Modifiers;

/// Represents a key event received from TSF or the keyboard hook.
#[derive(Clone, Debug)]
pub struct KeyEvent {
    pub keycode: u32,
    pub modifiers: Modifiers,
    pub is_key_down: bool,
}

impl KeyEvent {
    pub fn new(keycode: u32, modifiers: Modifiers, is_key_down: bool) -> Self {
        Self { keycode, modifiers, is_key_down }
    }

    /// Dispatch this key event to the Rime engine.
    /// Returns true if the engine handled the key.
    pub fn dispatch_to_engine(
        &self,
        engine: &skyme_rime_engine::Engine,
        session: &skyme_rime_engine::Session,
    ) -> bool {
        matches!(
            engine.process_key(session, self.keycode, self.modifiers),
            skyme_rime_engine::KeyProcessResult::Handled
        )
    }
}

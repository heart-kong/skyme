use crate::raw;

/// A handle to an active Rime input session.
///
/// Sessions are created by [`crate::Engine::create_session`].
/// They are automatically destroyed when dropped.
///
/// # Panics
///
/// If drop order is violated (e.g. the `Engine` is finalized before
/// all `Session` handles are dropped), `RimeDestroySession` will be
/// called after `RimeFinalize`, which is undefined behaviour in librime.
/// Ensure all sessions are dropped before their parent `Engine`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Session {
    id: u64,
}

impl Session {
    /// Create a new session from a raw librime session ID.
    ///
    /// This is called by `Engine::create_session`. Do not call directly.
    pub fn new(id: u64) -> Self {
        Self { id }
    }

    /// The raw librime session ID.
    pub fn id(&self) -> u64 {
        self.id
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        unsafe { raw::destroy_session(self.id) }
    }
}

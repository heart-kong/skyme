use std::sync::Arc;
use crate::ffi::RimeApi;

/// A handle to an active Rime input session.
///
/// Sessions are created by [`crate::Engine::create_session`].
/// They are automatically destroyed when dropped.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Session {
    id: u64,
}

impl Session {
    pub fn new(id: u64) -> Self { Self { id } }
    pub fn id(&self) -> u64 { self.id }
}

//! Shared types, EventBus, and common utilities for the Skyme input method.
//!
//! Every crate in this project depends on this crate for foundational types.
//! No crate should duplicate the types defined here.

pub mod event;
pub mod eventbus;
pub mod types;

pub use event::*;
pub use eventbus::*;
pub use types::*;

//! TSF (Text Services Framework) integration for Skyme.
//!
//! Handles all Windows TSF communication:
//! - Text service registration and COM lifecycle
//! - Composition management (start, end, cancel)
//! - Keyboard event processing
//! - Focus tracking

pub mod processor;
pub mod composition;
pub mod context;
pub mod document;
pub mod threadmgr;
pub mod keyevent;
pub mod com;
pub mod registrar;
pub mod error;

pub use processor::TextServiceProcessor;
pub use composition::CompositionManager;
pub use context::InputContext;
pub use document::DocumentManager;
pub use threadmgr::ThreadManager;
pub use keyevent::KeyEvent;
pub use registrar::ClsidRegistrar;
pub use error::ImeError;

pub const DISPLAY_NAME: &str = "Skyme Input Method";

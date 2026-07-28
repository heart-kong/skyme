//! TSF (Text Services Framework) integration for Skyme.
//!
//! Handles all Windows TSF communication:
//! - Text service registration and COM lifecycle
//! - Composition management (start, end, cancel)
//! - Keyboard event processing via ITfKeyEventSink
//! - Focus tracking via ITfThreadMgrEventSink
//!
//! No rendering — that is the responsibility of `candidate-ui`.

pub mod processor;
pub mod composition;
pub mod context;
pub mod document;
pub mod threadmgr;
pub mod keyevent;
pub mod com_service;
pub mod registrar;
pub mod error;

pub use processor::TextServiceProcessor;
pub use composition::CompositionManager;
pub use context::InputContext;
pub use document::DocumentManager;
pub use threadmgr::ThreadManager;
pub use keyevent::KeyEvent;
pub use com_service::SkymeTextService;
pub use registrar::ClsidRegistrar;
pub use error::ImeError;

/// The CLSID for the Skyme text service.
pub const CLSID_SKYME: &str = "{E4B5E5D0-1A2B-3C4D-5E6F-7890ABCDEF01}";

/// Profile GUID for the Skyme text service.
pub const PROFILE_GUID: &str = "{F1A2B3C4-D5E6-7890-ABCD-EF0123456789}";

/// The display name of the input method in the language bar.
pub const DISPLAY_NAME: &str = "Skyme Input Method";

/// The language ID (neutral — matches all locales).
pub const LANG_ID: u16 = 0x0409; // en-US for now, should be neutral

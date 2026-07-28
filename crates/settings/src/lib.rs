//! Settings management — handles hot-reloading of theme, IME, and UI configs.
//!
//! Watches configuration files on disk and emits [`ConfigReloaded`](skyme_common::event::Event::ConfigReloaded)
//! events when they change. No deploy step needed — changes take effect immediately.

pub mod theme;
pub mod watcher;

pub use theme::ThemeConfig;
pub use watcher::ConfigWatcher;

use skyme_common::event::Event;
use skyme_config::ConfigCenter;
use std::path::Path;

/// Aggregated runtime settings built from config files.
#[derive(Clone, Debug)]
pub struct Settings {
    pub theme: ThemeConfig,
}

impl Settings {
    pub fn new() -> Self {
        Self { theme: ThemeConfig::default() }
    }

    /// Load theme from a JSON file.
    pub fn load_theme(&mut self, path: &Path) -> Result<(), SettingsError> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            self.theme = serde_json::from_str(&content)?;
        }
        Ok(())
    }

    /// Poll for config changes and return an event if a reload occurred.
    pub fn poll_reload(&mut self, config: &mut ConfigCenter, engine: &skyme_rime_engine::Engine) -> Option<Event> {
        match config.try_reload(engine) {
            Ok(true) => Some(Event::ConfigReloaded),
            _ => None,
        }
    }
}

impl Default for Settings { fn default() -> Self { Self::new() } }

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SettingsError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
}

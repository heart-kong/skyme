//! Configuration centre — merges Rime YAML, UI config, and theme
//! into a single [`RuntimeConfig`].
//!
//! # Sources
//!
//! | Source | Path | Format |
//! |--------|------|--------|
//! | UI settings | `{user_dir}/ui.toml` | TOML |
//! | Theme | `{user_dir}/theme.json` | JSON |
//! | Rime schema | via `rime-engine` | YAML |

pub mod runtime;
pub mod error;

pub use error::ConfigError;
pub use runtime::*;

use skyme_rime_engine::Engine;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Central configuration registry.
pub struct ConfigCenter {
    user_dir: PathBuf,
    runtime: RuntimeConfig,
    ui_file: UiConfigFile,
    ui_path: PathBuf,
    theme_path: PathBuf,
    /// Cached file mtimes for detecting changes.
    ui_mtime: Option<SystemTime>,
    theme_mtime: Option<SystemTime>,
}

impl ConfigCenter {
    /// Create a new config centre for the given user data directory.
    pub fn new(user_dir: &Path) -> Self {
        Self {
            user_dir: user_dir.to_path_buf(),
            runtime: RuntimeConfig::default(),
            ui_file: UiConfigFile::default(),
            ui_path: user_dir.join("ui.toml"),
            theme_path: user_dir.join("theme.json"),
            ui_mtime: None,
            theme_mtime: None,
        }
    }

    /// Load all configuration from disk and merge into [`RuntimeConfig`].
    pub fn load(&mut self, engine: &Engine) -> Result<(), ConfigError> {
        self.load_ui()?;
        self.merge(engine);
        log::info!("Configuration loaded from {}", self.user_dir.display());
        Ok(())
    }

    /// Check for changes and hot-reload if needed. Returns `true` if anything changed.
    pub fn try_reload(&mut self, engine: &Engine) -> Result<bool, ConfigError> {
        let mut changed = false;

        let ui_path = self.ui_path.clone();
        if Self::file_check(&ui_path, &mut self.ui_mtime) {
            self.load_ui()?;
            changed = true;
        }

        let theme_path = self.theme_path.clone();
        if Self::file_check(&theme_path, &mut self.theme_mtime) {
            changed = true;
        }

        if changed {
            self.merge(engine);
            log::info!("Configuration hot-reloaded");
        }
        Ok(changed)
    }

    // ── accessors ──

    pub fn runtime(&self) -> &RuntimeConfig { &self.runtime }
    pub fn runtime_mut(&mut self) -> &mut RuntimeConfig { &mut self.runtime }
    pub fn ui_config(&self) -> &UiConfigFile { &self.ui_file }
    pub fn user_dir(&self) -> &Path { &self.user_dir }

    // ── internal ──

    fn load_ui(&mut self) -> Result<(), ConfigError> {
        if self.ui_path.exists() {
            let content = std::fs::read_to_string(&self.ui_path)?;
            self.ui_file = toml::from_str(&content)?;
            self.ui_mtime = self.ui_path.metadata().ok().and_then(|m| m.modified().ok());
        } else {
            self.ui_file = UiConfigFile::default();
        }
        Ok(())
    }

    fn merge(&mut self, _engine: &Engine) {
        self.runtime = self.ui_file.to_runtime();
    }

    /// Check if a file has changed by comparing its modification time.
    fn file_check(path: &Path, cached: &mut Option<SystemTime>) -> bool {
        let current = path.metadata().ok().and_then(|m| m.modified().ok());
        let changed = current.is_some() && current != *cached;
        if changed {
            *cached = current;
        }
        changed
    }
}

/// Startup helper.
impl ConfigCenter {
    pub fn initialize(
        user_dir: &Path,
        engine: &Engine,
    ) -> Result<Self, ConfigError> {
        let mut cc = Self::new(user_dir);
        cc.load(engine)?;
        Ok(cc)
    }
}

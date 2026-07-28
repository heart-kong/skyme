use skyme_common::DisplayMode;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct RuntimeConfig {
    pub display_mode: DisplayMode,
    pub auto_show: bool,
    pub page_size: u32,
    pub max_composition_length: u32,
}

// ── File-based config formats ─────────────────────────────────────────────

/// Mirrors `ui.toml` on disk.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UiConfigFile {
    #[serde(default = "default_display_mode")]
    pub display_mode: String,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    #[serde(default = "default_font_scale")]
    pub font_scale: f32,
    #[serde(default = "default_true")]
    pub auto_show: bool,
    #[serde(default = "default_max_composition")]
    pub max_composition_length: u32,
}

fn default_display_mode() -> String { "floating".into() }
fn default_page_size() -> u32 { 5 }
fn default_font_scale() -> f32 { 1.0 }
fn default_true() -> bool { true }
fn default_max_composition() -> u32 { 64 }

impl UiConfigFile {
    pub fn to_runtime(&self) -> RuntimeConfig {
        RuntimeConfig {
            display_mode: match self.display_mode.as_str() {
                "inline" => DisplayMode::Inline,
                "dock" => DisplayMode::Dock,
                "classic" => DisplayMode::Classic,
                _ => DisplayMode::Floating,
            },
            auto_show: self.auto_show,
            page_size: self.page_size,
            max_composition_length: self.max_composition_length,
        }
    }
}

impl Default for UiConfigFile {
    fn default() -> Self {
        Self {
            display_mode: default_display_mode(),
            page_size: default_page_size(),
            font_scale: default_font_scale(),
            auto_show: default_true(),
            max_composition_length: default_max_composition(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UiConfig { pub display_mode: DisplayMode, pub page_size: u32, pub font_scale: f32 }
impl Default for UiConfig { fn default() -> Self { Self { display_mode: DisplayMode::Floating, page_size: 5, font_scale: 1.0 } } }

#[derive(Clone, Debug)]
pub struct RimeConfig { pub default_schema: String }
impl Default for RimeConfig { fn default() -> Self { Self { default_schema: "luna_pinyin".into() } } }

#[derive(Clone, Debug)]
pub struct SessionConfig { pub ui: UiConfig, pub rime: RimeConfig }
impl Default for RuntimeConfig {
    fn default() -> Self {
        Self { display_mode: DisplayMode::Floating, auto_show: true, page_size: 5, max_composition_length: 64 }
    }
}

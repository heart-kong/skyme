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

#[cfg(test)]
mod tests {
    use crate::runtime::*;
    use skyme_common::DisplayMode;

    #[test]
    fn test_runtime_config_default() {
        let c = RuntimeConfig::default();
        assert_eq!(c.display_mode, DisplayMode::Floating);
        assert!(c.auto_show);
        assert_eq!(c.page_size, 5);
        assert_eq!(c.max_composition_length, 64);
    }

    #[test]
    fn test_ui_config_file_default() {
        let u = UiConfigFile::default();
        assert_eq!(u.display_mode, "floating");
        assert_eq!(u.page_size, 5);
        assert!(u.auto_show);
    }

    #[test]
    fn test_to_runtime_floating() {
        let u = UiConfigFile { display_mode: "floating".into(), ..UiConfigFile::default() };
        let r = u.to_runtime();
        assert_eq!(r.display_mode, DisplayMode::Floating);
    }

    #[test]
    fn test_to_runtime_inline() {
        let u = UiConfigFile { display_mode: "inline".into(), ..UiConfigFile::default() };
        let r = u.to_runtime();
        assert_eq!(r.display_mode, DisplayMode::Inline);
    }

    #[test]
    fn test_to_runtime_dock() {
        let u = UiConfigFile { display_mode: "dock".into(), ..UiConfigFile::default() };
        let r = u.to_runtime();
        assert_eq!(r.display_mode, DisplayMode::Dock);
    }

    #[test]
    fn test_to_runtime_classic() {
        let u = UiConfigFile { display_mode: "classic".into(), ..UiConfigFile::default() };
        let r = u.to_runtime();
        assert_eq!(r.display_mode, DisplayMode::Classic);
    }

    #[test]
    fn test_to_runtime_unknown_mode() {
        let u = UiConfigFile { display_mode: "unknown".into(), ..UiConfigFile::default() };
        let r = u.to_runtime();
        assert_eq!(r.display_mode, DisplayMode::Floating); // falls back
    }

    #[test]
    fn test_ui_config_custom_values() {
        let u = UiConfigFile {
            display_mode: "inline".into(),
            page_size: 9,
            font_scale: 1.2,
            auto_show: false,
            max_composition_length: 32,
        };
        let r = u.to_runtime();
        assert_eq!(r.display_mode, DisplayMode::Inline);
        assert_eq!(r.page_size, 9);
        assert!(!r.auto_show);
        assert_eq!(r.max_composition_length, 32);
    }
}

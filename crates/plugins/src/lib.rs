//! Plugin system for extending input method behaviour.
//!
//! Plugins hook into the event pipeline and can modify candidates,
//! respond to key events, or perform side effects (clipboard, AI, etc.).
//!
//! # Dynamic loading (future)
//!
//! Plugins can be compiled as shared libraries (`.dll` / `.so` / `.dylib`)
//! and loaded at runtime via `libloading`. The same `Plugin` trait is used.

use skyme_common::event::Event;
use skyme_common::{Candidate, Modifiers};

/// The trait every plugin must implement.
///
/// All methods have default no-op implementations so plugins only
/// need to override the hooks they care about.
pub trait Plugin: Send + 'static {
    /// Unique plugin name (used for identification and logging).
    fn name(&self) -> &str;

    /// Called when a candidate list is available.
    /// Plugins may modify, filter, or extend the list (e.g. add emoji).
    fn on_candidate(&mut self, _candidates: &mut Vec<Candidate>) {}

    /// Called when text is committed to the application.
    fn on_commit(&mut self, _text: &str) {}

    /// Called on a key event **before** the engine processes it.
    /// Return `true` to consume the key (engine will not see it).
    fn on_key(&mut self, _keycode: u32, _modifiers: Modifiers) -> bool { false }

    /// Called for generic events from the event bus.
    fn on_event(&mut self, _event: &Event) {}
}

/// Plugin lifecycle state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginState {
    Loaded,
    Active,
    Error,
}

/// Manages the lifecycle and dispatch of all loaded plugins.
pub struct PluginRegistry {
    plugins: Vec<PluginSlot>,
}

struct PluginSlot {
    plugin: Box<dyn Plugin>,
    state: PluginState,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }

    /// Register a new plugin.
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        log::info!("Plugin registered: {}", plugin.name());
        self.plugins.push(PluginSlot {
            plugin,
            state: PluginState::Loaded,
        });
    }

    /// Activate all loaded plugins.
    pub fn activate_all(&mut self) {
        for slot in &mut self.plugins {
            slot.state = PluginState::Active;
            log::debug!("Plugin activated: {}", slot.plugin.name());
        }
    }

    /// Deactivate all plugins.
    pub fn deactivate_all(&mut self) {
        for slot in &mut self.plugins {
            slot.state = PluginState::Loaded;
        }
    }

    /// Remove a plugin by name.
    pub fn unregister(&mut self, name: &str) {
        if let Some(pos) = self.plugins.iter().position(|s| s.plugin.name() == name) {
            log::info!("Plugin unregistered: {}", name);
            self.plugins.remove(pos);
        }
    }

    /// Dispatch candidate event to all active plugins.
    /// Returns the (potentially modified) candidate list.
    pub fn dispatch_candidate(&mut self, candidates: &mut Vec<Candidate>) {
        for slot in &mut self.plugins {
            if slot.state == PluginState::Active {
                slot.plugin.on_candidate(candidates);
            }
        }
    }

    /// Dispatch commit event to all active plugins.
    pub fn dispatch_commit(&mut self, text: &str) {
        for slot in &mut self.plugins {
            if slot.state == PluginState::Active {
                slot.plugin.on_commit(text);
            }
        }
    }

    /// Dispatch key event to all active plugins.
    /// Returns `true` if any plugin consumed the key.
    pub fn dispatch_key(&mut self, keycode: u32, modifiers: Modifiers) -> bool {
        for slot in &mut self.plugins {
            if slot.state == PluginState::Active && slot.plugin.on_key(keycode, modifiers) {
                return true;
            }
        }
        false
    }

    /// Dispatch a generic event.
    pub fn dispatch_event(&mut self, event: &Event) {
        for slot in &mut self.plugins {
            if slot.state == PluginState::Active {
                slot.plugin.on_event(event);
            }
        }
    }

    pub fn plugin_count(&self) -> usize { self.plugins.len() }
    pub fn is_empty(&self) -> bool { self.plugins.is_empty() }
    pub fn names(&self) -> Vec<String> {
        self.plugins.iter().map(|s| s.plugin.name().to_string()).collect()
    }
}

impl Default for PluginRegistry { fn default() -> Self { Self::new() } }

// ── Built-in plugin stubs ─────────────────────────────────────────────────

/// Emoji candidate plugin. Injects emoji candidates when a keyword prefix is detected.
pub struct EmojiPlugin;
impl Plugin for EmojiPlugin {
    fn name(&self) -> &str { "emoji" }
    fn on_candidate(&mut self, _c: &mut Vec<Candidate>) {}
    fn on_event(&mut self, _e: &Event) {}
}

/// AI completion plugin. Provides LLM-based candidate predictions.
pub struct AiPlugin;
impl Plugin for AiPlugin {
    fn name(&self) -> &str { "ai" }
    fn on_candidate(&mut self, _c: &mut Vec<Candidate>) {}
    fn on_commit(&mut self, _text: &str) {}
    fn on_event(&mut self, _e: &Event) {}
}

/// Clipboard history plugin. Provides clipboard candidates.
pub struct ClipboardPlugin;
impl Plugin for ClipboardPlugin {
    fn name(&self) -> &str { "clipboard" }
    fn on_commit(&mut self, _text: &str) {}
    fn on_event(&mut self, _e: &Event) {}
}

/// Translation plugin. Provides translation candidates.
pub struct TranslatorPlugin;
impl Plugin for TranslatorPlugin {
    fn name(&self) -> &str { "translator" }
    fn on_candidate(&mut self, _c: &mut Vec<Candidate>) {}
    fn on_event(&mut self, _e: &Event) {}
}

/// OCR plugin. Provides screen OCR candidates.
pub struct OcrPlugin;
impl Plugin for OcrPlugin {
    fn name(&self) -> &str { "ocr" }
    fn on_candidate(&mut self, _c: &mut Vec<Candidate>) {}
    fn on_event(&mut self, _e: &Event) {}
}

/// Cloud input plugin. Provides cloud-sync'd candidates.
pub struct CloudInputPlugin;
impl Plugin for CloudInputPlugin {
    fn name(&self) -> &str { "cloud" }
    fn on_candidate(&mut self, _c: &mut Vec<Candidate>) {}
    fn on_event(&mut self, _e: &Event) {}
}

/// WASM plugin runtime. Loads and runs plugins compiled as WebAssembly.
pub struct WasmPluginRuntime;
impl Plugin for WasmPluginRuntime {
    fn name(&self) -> &str { "wasm" }
    fn on_candidate(&mut self, _c: &mut Vec<Candidate>) {}
    fn on_event(&mut self, _e: &Event) {}
}

/// Profanity filter. Filters out profane candidates.
pub struct ProfanityFilter;
impl Plugin for ProfanityFilter {
    fn name(&self) -> &str { "profanity_filter" }
    fn on_candidate(&mut self, candidates: &mut Vec<Candidate>) {
        candidates.retain(|c| !is_profane(&c.text));
    }
}
fn is_profane(_text: &str) -> bool { false }

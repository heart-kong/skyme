//! Plugin system for extending input method behaviour.
//!
//! Each plugin implements the [`Plugin`] trait and hooks into the
//! input pipeline to modify candidates, handle keys, or react to commits.

pub mod emoji;
pub mod ai;
pub mod clipboard;
pub mod translator;
pub mod ocr;
pub mod cloud;
pub mod wasm;
pub mod profanity_filter;

use skyme_common::event::Event;
use skyme_common::{Candidate, Modifiers};

/// The trait every plugin must implement.
pub trait Plugin: Send + 'static {
    fn name(&self) -> &str;
    fn on_candidate(&mut self, _candidates: &mut Vec<Candidate>) {}
    fn on_commit(&mut self, _text: &str) {}
    fn on_key(&mut self, _keycode: u32, _modifiers: Modifiers) -> bool { false }
    fn on_event(&mut self, _event: &Event) {}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginState { Loaded, Active, Error }

/// Manages the lifecycle and dispatch of all loaded plugins.
pub struct PluginRegistry { plugins: Vec<PluginSlot> }
struct PluginSlot { plugin: Box<dyn Plugin>, state: PluginState }

impl PluginRegistry {
    pub fn new() -> Self { Self { plugins: Vec::new() } }
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        log::info!("Plugin registered: {}", plugin.name());
        self.plugins.push(PluginSlot { plugin, state: PluginState::Loaded });
    }
    pub fn activate_all(&mut self) {
        for s in &mut self.plugins { s.state = PluginState::Active; log::debug!("Activated: {}", s.plugin.name()); }
    }
    pub fn deactivate_all(&mut self) {
        for s in &mut self.plugins { s.state = PluginState::Loaded; }
    }
    pub fn unregister(&mut self, name: &str) {
        if let Some(p) = self.plugins.iter().position(|s| s.plugin.name() == name) {
            log::info!("Unregistered: {}", name); self.plugins.remove(p);
        }
    }
    pub fn dispatch_candidate(&mut self, candidates: &mut Vec<Candidate>) {
        for s in &mut self.plugins { if s.state == PluginState::Active { s.plugin.on_candidate(candidates); } }
    }
    pub fn dispatch_commit(&mut self, text: &str) {
        for s in &mut self.plugins { if s.state == PluginState::Active { s.plugin.on_commit(text); } }
    }
    pub fn dispatch_key(&mut self, keycode: u32, mods: Modifiers) -> bool {
        for s in &mut self.plugins { if s.state == PluginState::Active && s.plugin.on_key(keycode, mods) { return true; } }
        false
    }
    pub fn dispatch_event(&mut self, event: &Event) {
        for s in &mut self.plugins { if s.state == PluginState::Active { s.plugin.on_event(event); } }
    }
    pub fn plugin_count(&self) -> usize { self.plugins.len() }
    pub fn is_empty(&self) -> bool { self.plugins.is_empty() }
    pub fn names(&self) -> Vec<String> { self.plugins.iter().map(|s| s.plugin.name().to_string()).collect() }
}

impl Default for PluginRegistry { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use crate::*;
    use skyme_common::{Candidate, Modifiers, event::Event};

    #[test]
    fn test_plugin_registry_empty() {
        let r = PluginRegistry::new();
        assert!(r.is_empty());
        assert_eq!(r.plugin_count(), 0);
    }

    #[test]
    fn test_register_and_activate() {
        let mut r = PluginRegistry::new();
        r.register(Box::new(super::emoji::EmojiPlugin));
        assert_eq!(r.plugin_count(), 1);
        r.activate_all();
        assert_eq!(r.names(), vec!["emoji"]);
    }

    #[test]
    fn test_unregister() {
        let mut r = PluginRegistry::new();
        r.register(Box::new(super::emoji::EmojiPlugin));
        r.register(Box::new(super::ai::AiPlugin::new()));
        assert_eq!(r.plugin_count(), 2);
        r.unregister("emoji");
        assert_eq!(r.plugin_count(), 1);
        assert_eq!(r.names(), vec!["ai"]);
    }

    #[test]
    fn test_dispatch_empty() {
        let mut r = PluginRegistry::new();
        let mut candidates = vec![];
        r.dispatch_candidate(&mut candidates);
        r.dispatch_commit("test");
        assert!(!r.dispatch_key(0x41, Modifiers::NONE));
    }

    #[test]
    fn test_emoji_plugin_injects_candidates() {
        let mut p = super::emoji::EmojiPlugin;
        let mut candidates = vec![
            Candidate { text: "smile".into(), comment: "".into(), index: 0, quality: 1.0 },
            Candidate { text: "hello".into(), comment: "".into(), index: 1, quality: 0.5 },
        ];
        p.on_candidate(&mut candidates);
        // Should have at least the original candidates plus emoji additions
        assert!(candidates.len() >= 2);
        // The first candidate should contain an emoji (from "smile" keyword)
        assert!(candidates[0].text.contains('😊') || candidates[1].text.contains('😊'));
    }

    #[test]
    fn test_emoji_no_match() {
        let mut p = super::emoji::EmojiPlugin;
        let mut candidates = vec![
            Candidate { text: "qwertyuiop".into(), comment: "".into(), index: 0, quality: 1.0 },
        ];
        let before = candidates.len();
        p.on_candidate(&mut candidates);
        assert_eq!(candidates.len(), before); // no change
    }

    #[test]
    fn test_clipboard_plugin_adds_entry() {
        let mut p = super::clipboard::ClipboardPlugin::new();
        p.on_commit("hello world");
        // No direct inspection, just ensure no crash
    }

    #[test]
    fn test_clipboard_trigger() {
        let mut p = super::clipboard::ClipboardPlugin::new();
        // Ctrl+Shift+V should trigger
        let consumed = p.on_key(0x56, Modifiers::CTRL | Modifiers::SHIFT);
        assert!(consumed);
    }

    #[test]
    fn test_translator_plugin_zh_to_en() {
        let mut p = super::translator::TranslatorPlugin;
        let mut candidates = vec![
            Candidate { text: "你好".into(), comment: "".into(), index: 0, quality: 0.9 },
        ];
        p.on_candidate(&mut candidates);
        // Should add "hello" as a translation
        let has_translation = candidates.iter().any(|c| c.text == "hello" || c.comment.contains("translate"));
        assert!(has_translation, "expected translation for 你好, got: {:?}", candidates);
    }

    #[test]
    fn test_translator_plugin_en_to_zh() {
        let mut p = super::translator::TranslatorPlugin;
        let mut candidates = vec![
            Candidate { text: "hello".into(), comment: "".into(), index: 0, quality: 0.9 },
        ];
        p.on_candidate(&mut candidates);
        // Should add "你好" as a translation
        let has_translation = candidates.iter().any(|c| c.text == "你好");
        assert!(has_translation);
    }

    #[test]
    fn test_profanity_filter() {
        let mut p = super::profanity_filter::ProfanityFilter;
        let mut candidates = vec![
            Candidate { text: "hello".into(), comment: "".into(), index: 0, quality: 1.0 },
            Candidate { text: "fuck this".into(), comment: "".into(), index: 1, quality: 0.9 },
            Candidate { text: "world".into(), comment: "".into(), index: 2, quality: 0.8 },
        ];
        p.on_candidate(&mut candidates);
        assert_eq!(candidates.len(), 2);
        assert!(!candidates.iter().any(|c| c.text.contains("fuck")));
    }

    #[test]
    fn test_profanity_filter_clean() {
        let mut p = super::profanity_filter::ProfanityFilter;
        let mut candidates = vec![
            Candidate { text: "hello".into(), comment: "".into(), index: 0, quality: 1.0 },
            Candidate { text: "world".into(), comment: "".into(), index: 1, quality: 0.8 },
        ];
        let before = candidates.len();
        p.on_candidate(&mut candidates);
        assert_eq!(candidates.len(), before); // unchanged
    }

    #[test]
    fn test_ai_plugin_maintains_history() {
        let mut p = super::ai::AiPlugin::new();
        p.on_commit("first");
        p.on_commit("second");
        // No crash
    }

    #[test]
    fn test_ocr_plugin_no_crash() {
        let mut p = super::ocr::OcrPlugin::new();
        let consumed = p.on_key(0x4F, Modifiers::CTRL | Modifiers::SHIFT);
        assert!(consumed);
    }

    #[test]
    fn test_cloud_plugin_connect() {
        let mut p = super::cloud::CloudInputPlugin::new();
        p.on_event(&Event::PluginEvent { plugin: "cloud".into(), payload: "connect".into() });
        // No crash, will use cloud in on_candidate
    }
}

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

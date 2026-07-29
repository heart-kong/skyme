//! Clipboard history plugin.
//!
//! Monitors clipboard content and provides recent clipboard entries
//! as candidates when triggered (e.g., by a key chord).

use crate::Plugin;
use skyme_common::{Candidate, Modifiers, event::Event};
use std::collections::VecDeque;

const MAX_ENTRIES: usize = 20;
const TRIGGER_KEY: u32 = 0x56; // V

pub struct ClipboardPlugin {
    /// Recent clipboard entries (newest first).
    entries: VecDeque<String>,
    /// Whether trigger key is held.
    triggered: bool,
}

impl ClipboardPlugin {
    pub fn new() -> Self {
        Self { entries: VecDeque::with_capacity(MAX_ENTRIES), triggered: false }
    }

    fn add_entry(&mut self, text: &str) {
        if text.is_empty() || text.len() > 1024 { return; }
        // Avoid duplicates
        if let Some(front) = self.entries.front() {
            if front == text { return; }
        }
        self.entries.push_front(text.to_owned());
        if self.entries.len() > MAX_ENTRIES { self.entries.pop_back(); }
    }
}

impl Default for ClipboardPlugin { fn default() -> Self { Self::new() } }

impl Plugin for ClipboardPlugin {
    fn name(&self) -> &str { "clipboard" }

    fn on_key(&mut self, keycode: u32, modifiers: Modifiers) -> bool {
        // Ctrl+Shift+V triggers clipboard history
        if keycode == TRIGGER_KEY && modifiers.bits() & 0x03 == 0x03 {
            self.triggered = !self.triggered;
            log::info!("Clipboard plugin: triggered={}", self.triggered);
            true // consume the key
        } else {
            self.triggered = false;
            false
        }
    }

    fn on_commit(&mut self, text: &str) {
        if !text.is_empty() && text.len() < 1024 {
            self.add_entry(text);
        }
    }

    fn on_candidate(&mut self, candidates: &mut Vec<Candidate>) {
        if !self.triggered || self.entries.is_empty() { return; }

        // Inject clipboard entries as candidates
        let clipboard_candidates: Vec<Candidate> = self.entries.iter().enumerate().map(|(i, text)| {
            let display = if text.len() > 40 {
                format!("{}...", &text[..37])
            } else {
                text.clone()
            };
            Candidate {
                text: text.clone(),
                comment: format!("📋 #{}", i + 1),
                index: i as u32,
                quality: 100.0 - i as f64,
            }
        }).collect();

        // Prepend clipboard candidates
        let mut result = clipboard_candidates;
        result.extend(candidates.drain(..));
        *candidates = result;

        self.triggered = false; // one-shot trigger
    }

    fn on_event(&mut self, event: &Event) {
        if let Event::PluginEvent { plugin, payload } = event {
            if plugin == "clipboard" {
                self.add_entry(payload);
            }
        }
    }
}

//! AI completion plugin.
//!
//! Provides LLM-based candidate predictions and text completion.
//! This is a scaffold — connect to a local LLM (e.g., llama.cpp)
//! or cloud API (e.g., OpenAI) for actual functionality.

use crate::Plugin;
use skyme_common::{Candidate, event::Event};
use std::collections::VecDeque;

const MAX_HISTORY: usize = 100;

pub struct AiPlugin {
    /// Recent commit history for context.
    history: VecDeque<String>,
    /// Whether the AI backend is available.
    connected: bool,
}

impl AiPlugin {
    pub fn new() -> Self {
        Self { history: VecDeque::with_capacity(MAX_HISTORY), connected: false }
    }

    /// Predict next word / completion based on context.
    /// Stub — replace with actual LLM call.
    fn predict(&self, _context: &[String]) -> Vec<Candidate> {
        if !self.connected { return Vec::new(); }
        // Placeholder: would call LLM API here
        Vec::new()
    }
}

impl Default for AiPlugin { fn default() -> Self { Self::new() } }

impl Plugin for AiPlugin {
    fn name(&self) -> &str { "ai" }

    fn on_commit(&mut self, text: &str) {
        self.history.push_back(text.to_owned());
        if self.history.len() > MAX_HISTORY {
            self.history.pop_front();
        }
    }

    fn on_candidate(&mut self, candidates: &mut Vec<Candidate>) {
        let predictions = self.predict(
            &self.history.iter().rev().take(5).cloned().collect::<Vec<_>>()
        );
        if !predictions.is_empty() {
            candidates.extend(predictions);
        }
    }

    fn on_event(&mut self, event: &Event) {
        if let Event::PluginEvent { plugin, payload } = event {
            if plugin == "ai" && payload == "connect" {
                self.connected = true;
                log::info!("AI plugin connected");
            }
        }
    }
}

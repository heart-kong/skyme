//! Cloud input plugin.
//!
//! Provides cloud-synchronized candidates across devices.
//! This is a scaffold for future implementation with a sync backend.

use crate::Plugin;
use skyme_common::{Candidate, Modifiers, event::Event};
use std::collections::HashSet;

pub struct CloudInputPlugin {
    connected: bool,
    /// User's personalized candidates from the cloud.
    cloud_candidates: Vec<(String, f64)>, // (text, frequency)
    /// Recently committed words to sync.
    pending_sync: Vec<String>,
}

impl CloudInputPlugin {
    pub fn new() -> Self {
        Self { connected: false, cloud_candidates: Vec::new(), pending_sync: Vec::new() }
    }

    /// Sync pending commits to the cloud.
    /// Stub — would send to a sync server.
    fn sync_to_cloud(&mut self) {
        if !self.connected || self.pending_sync.is_empty() { return; }
        log::info!("Cloud sync: {} entries", self.pending_sync.len());
        self.pending_sync.clear();
    }

    /// Fetch personalized candidates from the cloud.
    /// Stub — would query a sync server.
    fn fetch_cloud_candidates(&mut self, _context: &str) -> Vec<(String, f64)> {
        if !self.connected { return Vec::new(); }
        self.cloud_candidates.clone()
    }
}

impl Default for CloudInputPlugin { fn default() -> Self { Self::new() } }

impl Plugin for CloudInputPlugin {
    fn name(&self) -> &str { "cloud" }

    fn on_commit(&mut self, text: &str) {
        if text.len() > 2 && text.len() < 200 {
            self.pending_sync.push(text.to_owned());
            if self.pending_sync.len() >= 10 {
                self.sync_to_cloud();
            }
        }
    }

    fn on_candidate(&mut self, candidates: &mut Vec<Candidate>) {
        let cloud = self.fetch_cloud_candidates("");
        if cloud.is_empty() { return; }

        for (text, freq) in &cloud {
            candidates.push(Candidate {
                text: text.clone(),
                comment: "☁️ cloud".into(),
                index: 0, quality: *freq,
            });
        }
    }

    fn on_event(&mut self, event: &Event) {
        match event {
            Event::PluginEvent { plugin, payload } if plugin == "cloud" => {
                if payload == "connect" { self.connected = true; log::info!("Cloud connected"); }
                if payload == "disconnect" { self.connected = false; }
            }
            _ => {}
        }
    }
}

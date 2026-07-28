//! Developer diagnostics panel — similar to Chrome DevTools for the input method.
//!
//! Provides real-time visibility into:
//! - TSF events (composition, commit, focus)
//! - Candidate state
//! - Key events
//! - Engine status and schema info
//! - Performance (FPS, latency, memory)
//!
//! Accessible via a configurable keyboard shortcut (default: Ctrl+Shift+F12).

pub mod inspector;
pub mod metrics;

pub use inspector::{Inspector, InspectorData, SessionInfo, CompositionInfo, EngineInfo, PerformanceInfo};
pub use metrics::{MetricsCollector, PerfMetrics};

use skyme_common::event::Event;
use skyme_common::EventListener;

/// Developer diagnostics console.
///
/// Listens to all system events and maintains real-time state snapshots
/// that can be inspected through the overlay UI.
pub struct DiagnosticsConsole {
    event_log: Vec<Event>,
    metrics: MetricsCollector,
    inspector: Inspector,
}

impl DiagnosticsConsole {
    pub fn new() -> Self {
        Self {
            event_log: Vec::with_capacity(1024),
            metrics: MetricsCollector::new(),
            inspector: Inspector::new(),
        }
    }

    pub fn record_event(&mut self, event: &Event) {
        if self.event_log.len() >= 1024 { self.event_log.remove(0); }
        self.event_log.push(event.clone());
        self.inspector.on_event(event);
        self.metrics.record_event(event);
    }

    pub fn event_log(&self) -> &[Event] { &self.event_log }
    pub fn metrics(&self) -> &MetricsCollector { &self.metrics }
    pub fn inspector(&self) -> &Inspector { &self.inspector }
    pub fn inspector_mut(&mut self) -> &mut Inspector { &mut self.inspector }
    pub fn is_visible(&self) -> bool { self.inspector.visible }

    pub fn toggle(&mut self) {
        self.inspector.visible = !self.inspector.visible;
        log::info!("Diagnostics panel {}", if self.inspector.visible { "shown" } else { "hidden" });
    }

    /// Build the current state snapshot for rendering.
    pub fn snapshot(&self) -> InspectorData {
        self.inspector.snapshot(&self.metrics)
    }
}

impl Default for DiagnosticsConsole { fn default() -> Self { Self::new() } }

impl EventListener for DiagnosticsConsole {
    fn on_event(&mut self, event: &Event) { self.record_event(event); }
}

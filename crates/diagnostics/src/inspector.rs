//! Inspector overlay — displays real-time state snapshots.

use crate::metrics::MetricsCollector;
use skyme_common::event::Event;

/// The inspector overlay state.
pub struct Inspector {
    pub visible: bool,
    pub current_session_id: u64,
    pub current_composition: String,
    pub current_schema: String,
    pub candidate_count: usize,
    pub last_commit: String,
    pub fps: f64,
    pub latency_ms: f64,
}

impl Inspector {
    pub fn new() -> Self {
        Self {
            visible: false, current_session_id: 0, current_composition: String::new(),
            current_schema: String::new(), candidate_count: 0, last_commit: String::new(),
            fps: 0.0, latency_ms: 0.0,
        }
    }

    pub fn on_event(&mut self, event: &Event) {
        match event {
            Event::CompositionUpdated { session_id, preedit, .. } => {
                self.current_session_id = *session_id;
                self.current_composition = preedit.clone();
            }
            Event::Commit { text } => {
                self.last_commit = text.clone();
            }
            Event::CandidateChanged { .. } => {
                self.candidate_count += 1;
            }
            _ => {}
        }
    }

    pub fn toggle(&mut self) { self.visible = !self.visible; }

    /// Build a structured snapshot from current state + metrics.
    pub fn snapshot(&self, metrics: &MetricsCollector) -> InspectorData {
        InspectorData {
            session: SessionInfo { id: self.current_session_id, active: !self.current_composition.is_empty(), schema: self.current_schema.clone() },
            composition: CompositionInfo { preedit: self.current_composition.clone(), cursor: 0, candidates: self.candidate_count, page: 0 },
            engine: EngineInfo { initialized: true, memory_kb: 0 },
            performance: PerformanceInfo { fps: metrics.fps(), latency_ms: metrics.avg_latency_ms() },
        }
    }
}

/// Data rendered in the inspector overlay.
#[derive(Clone, Debug)]
pub struct InspectorData {
    pub session: SessionInfo,
    pub composition: CompositionInfo,
    pub engine: EngineInfo,
    pub performance: PerformanceInfo,
}

#[derive(Clone, Debug, Default)]
pub struct SessionInfo { pub id: u64, pub active: bool, pub schema: String }
#[derive(Clone, Debug, Default)]
pub struct CompositionInfo { pub preedit: String, pub cursor: usize, pub candidates: usize, pub page: u32 }
#[derive(Clone, Debug, Default)]
pub struct EngineInfo { pub initialized: bool, pub memory_kb: u64 }
#[derive(Clone, Debug, Default)]
pub struct PerformanceInfo { pub fps: f64, pub latency_ms: f64 }

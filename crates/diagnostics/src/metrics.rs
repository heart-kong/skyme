//! Performance metrics collection.

use skyme_common::event::Event;
use std::time::Instant;

/// Collects performance metrics for the diagnostics panel.
pub struct MetricsCollector {
    frame_count: u64,
    last_fps_sample: Instant,
    current_fps: f64,
    event_count: u64,
    total_latency_ns: u64,
    latency_samples: u64,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            frame_count: 0, last_fps_sample: Instant::now(), current_fps: 0.0,
            event_count: 0, total_latency_ns: 0, latency_samples: 0,
        }
    }

    pub fn tick_frame(&mut self) {
        self.frame_count += 1;
        let elapsed = self.last_fps_sample.elapsed();
        if elapsed.as_secs_f64() >= 1.0 {
            self.current_fps = self.frame_count as f64 / elapsed.as_secs_f64();
            self.frame_count = 0;
            self.last_fps_sample = Instant::now();
        }
    }

    pub fn record_event(&mut self, _event: &Event) { self.event_count += 1; }

    pub fn record_latency(&mut self, latency_ns: u64) {
        self.total_latency_ns += latency_ns;
        self.latency_samples += 1;
    }

    pub fn fps(&self) -> f64 { self.current_fps }
    pub fn avg_latency_ms(&self) -> f64 {
        if self.latency_samples == 0 { 0.0 }
        else { (self.total_latency_ns as f64 / self.latency_samples as f64) / 1_000_000.0 }
    }
    pub fn event_count(&self) -> u64 { self.event_count }
    pub fn memory_kb(&self) -> u64 { 0 } // stub
}

impl Default for MetricsCollector { fn default() -> Self { Self::new() } }

#[derive(Clone, Debug)]
pub struct PerfMetrics { pub fps: f64, pub avg_latency_ms: f64, pub event_count: u64, pub memory_kb: u64 }

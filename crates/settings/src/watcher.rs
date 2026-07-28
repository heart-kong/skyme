//! File-system watcher for hot-reloading configuration files.
//!
//! Uses the `notify` crate to watch config directories and debounces
//! rapid change events into a single reload signal.

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// A debounced file watcher for config files.
pub struct ConfigWatcher {
    tx: Option<mpsc::Sender<()>>,
    dirty: Arc<Mutex<bool>>,
}

impl ConfigWatcher {
    pub fn new() -> Self {
        Self { tx: None, dirty: Arc::new(Mutex::new(false)) }
    }

    /// Start watching a directory for changes.
    pub fn start_watching(&mut self, dir: &Path) -> Result<(), String> {
        let dirty_clone = self.dirty.clone();
        let (tx, rx) = mpsc::channel::<()>();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    match event.kind {
                        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
                            *dirty_clone.lock().unwrap() = true;
                        }
                        _ => {}
                    }
                }
            },
            Config::default(),
        )
        .map_err(|e| format!("Failed to create file watcher: {}", e))?;

        watcher
            .watch(dir, RecursiveMode::NonRecursive)
            .map_err(|e| format!("Failed to watch {}: {}", dir.display(), e))?;

        self.tx = Some(tx);
        let debounce_dirty = self.dirty.clone();

        // Spawn debounce thread.
        std::thread::spawn(move || {
            loop {
                if rx.recv_timeout(Duration::from_secs(1)).is_ok() {}
                std::thread::sleep(Duration::from_millis(300));
                *debounce_dirty.lock().unwrap() = false;
            }
        });

        log::info!("Watching config directory: {}", dir.display());
        Ok(())
    }

    pub fn is_dirty(&self) -> bool { *self.dirty.lock().unwrap() }
    pub fn mark_dirty(&self) { *self.dirty.lock().unwrap() = true; }

    pub fn request_reload(&self) {
        if let Some(ref tx) = self.tx { let _ = tx.send(()); }
    }
}

impl Default for ConfigWatcher { fn default() -> Self { Self::new() } }

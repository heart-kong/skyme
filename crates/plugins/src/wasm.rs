//! WASM plugin runtime.
//!
//! Loads and runs plugins compiled as WebAssembly modules.
//! This is a scaffold — uses a simple plugin descriptor format
//! and delegates to an embedded WASM runtime (e.g., wasmtime/wasmi).

use crate::Plugin;
use skyme_common::{Candidate, Modifiers, event::Event};
use std::collections::HashMap;

/// Descriptor for a loaded WASM plugin.
struct WasmPluginDescriptor {
    name: String,
    path: String,
    // In production: store wasmtime::Store, Instance, etc.
}

pub struct WasmPluginRuntime {
    /// Loaded WASM plugins.
    plugins: HashMap<String, WasmPluginDescriptor>,
    /// Base directory for WASM modules.
    plugins_dir: String,
}

impl WasmPluginRuntime {
    pub fn new(plugins_dir: &str) -> Self {
        Self { plugins: HashMap::new(), plugins_dir: plugins_dir.to_owned() }
    }

    /// Load a WASM module from a file.
    /// Stub — would use wasmtime to compile and instantiate.
    pub fn load_plugin(&mut self, name: &str, filename: &str) -> Result<(), String> {
        let path = format!("{}/{}", self.plugins_dir, filename);
        if !std::path::Path::new(&path).exists() {
            return Err(format!("WASM plugin not found: {}", path));
        }

        log::info!("WASM plugin loaded: {} ({})", name, path);
        self.plugins.insert(name.to_owned(), WasmPluginDescriptor {
            name: name.to_owned(), path,
        });
        Ok(())
    }

    /// Call the `on_candidate` export of a WASM plugin.
    fn call_on_candidate(&self, _name: &str, _candidates: &mut Vec<Candidate>) {
        // TODO: call wasm export
    }
}

impl Default for WasmPluginRuntime { fn default() -> Self { Self::new("./plugins") } }

impl Plugin for WasmPluginRuntime {
    fn name(&self) -> &str { "wasm" }

    fn on_candidate(&mut self, candidates: &mut Vec<Candidate>) {
        for name in self.plugins.keys() {
            self.call_on_candidate(name, candidates);
        }
    }

    fn on_event(&mut self, event: &Event) {
        if let Event::PluginEvent { plugin, payload } = event {
            if plugin == "wasm" && payload.starts_with("load:") {
                let parts: Vec<&str> = payload.split(':').collect();
                if parts.len() >= 3 {
                    let _ = self.load_plugin(parts[1], parts[2]);
                }
            }
        }
    }
}

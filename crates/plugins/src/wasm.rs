use crate::Plugin;
use skyme_common::{Candidate, event::Event};

pub struct WasmPluginRuntime;
impl Plugin for WasmPluginRuntime {
    fn name(&self) -> &str { "wasm" }
    fn on_candidate(&mut self, _c: &mut Vec<Candidate>) {}
    fn on_event(&mut self, _e: &Event) {}
}

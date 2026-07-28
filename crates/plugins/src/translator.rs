use crate::Plugin;
use skyme_common::{Candidate, event::Event};

pub struct TranslatorPlugin;
impl Plugin for TranslatorPlugin {
    fn name(&self) -> &str { "translator" }
    fn on_candidate(&mut self, _c: &mut Vec<Candidate>) {}
    fn on_event(&mut self, _e: &Event) {}
}

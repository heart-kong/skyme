use crate::Plugin;
use skyme_common::{Candidate, event::Event};

pub struct AiPlugin;
impl Plugin for AiPlugin {
    fn name(&self) -> &str { "ai" }
    fn on_candidate(&mut self, _c: &mut Vec<Candidate>) {}
    fn on_commit(&mut self, _text: &str) {}
    fn on_event(&mut self, _e: &Event) {}
}

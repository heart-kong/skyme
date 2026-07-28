use crate::Plugin;
use skyme_common::{Candidate, event::Event};

pub struct CloudInputPlugin;
impl Plugin for CloudInputPlugin {
    fn name(&self) -> &str { "cloud" }
    fn on_candidate(&mut self, _c: &mut Vec<Candidate>) {}
    fn on_event(&mut self, _e: &Event) {}
}

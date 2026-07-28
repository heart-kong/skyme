use crate::Plugin;
use skyme_common::{Candidate, event::Event};

pub struct EmojiPlugin;
impl Plugin for EmojiPlugin {
    fn name(&self) -> &str { "emoji" }
    fn on_candidate(&mut self, _c: &mut Vec<Candidate>) {}
    fn on_event(&mut self, _e: &Event) {}
}

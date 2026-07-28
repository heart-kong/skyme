use crate::Plugin;
use skyme_common::event::Event;

pub struct ClipboardPlugin;
impl Plugin for ClipboardPlugin {
    fn name(&self) -> &str { "clipboard" }
    fn on_commit(&mut self, _text: &str) {}
    fn on_event(&mut self, _e: &Event) {}
}

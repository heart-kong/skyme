use crate::Plugin;
use skyme_common::{Candidate, event::Event};

pub struct OcrPlugin;
impl Plugin for OcrPlugin {
    fn name(&self) -> &str { "ocr" }
    fn on_candidate(&mut self, _c: &mut Vec<Candidate>) {}
    fn on_event(&mut self, _e: &Event) {}
}

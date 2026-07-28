use crate::Plugin;
use skyme_common::Candidate;

pub struct ProfanityFilter;
impl Plugin for ProfanityFilter {
    fn name(&self) -> &str { "profanity_filter" }
    fn on_candidate(&mut self, candidates: &mut Vec<Candidate>) {
        candidates.retain(|c| !is_profane(&c.text));
    }
}
fn is_profane(_text: &str) -> bool { false }

//! Profanity filter plugin.
//!
//! Filters out profane or unwanted candidates from the candidate list.

use crate::Plugin;
use skyme_common::Candidate;

/// List of profane words to filter.
const PROFANE_WORDS: &[&str] = &[
    "fuck", "shit", "damn", "ass", "bitch", "bastard", "crap",
    "dick", "piss", "slut", "whore", "cock", "cunt", "douche",
];

pub struct ProfanityFilter;

impl Plugin for ProfanityFilter {
    fn name(&self) -> &str { "profanity_filter" }

    fn on_candidate(&mut self, candidates: &mut Vec<Candidate>) {
        candidates.retain(|c| !is_profane(&c.text));
    }
}

fn is_profane(text: &str) -> bool {
    let lower = text.to_lowercase();
    for &word in PROFANE_WORDS {
        if lower.contains(word) {
            return true;
        }
    }
    false
}

//! Emoji candidate plugin.
//!
//! Scans candidate text for emoji keywords (e.g., "smile", "heart")
//! and injects corresponding emoji candidates at the top of the list.

use crate::Plugin;
use skyme_common::Candidate;

/// Emoji keyword → emoji mapping.
const EMOJI_TABLE: &[(&str, &str)] = &[
    ("smile", "😊"), ("happy", "😄"), ("laugh", "😂"), ("wink", "😉"),
    ("sad", "😢"), ("cry", "😭"), ("angry", "😠"), ("love", "❤️"),
    ("heart", "❤️"), ("kiss", "😘"), ("cool", "😎"), ("wow", "😮"),
    ("fire", "🔥"), ("star", "⭐"), ("moon", "🌙"), ("sun", "☀️"),
    ("cat", "🐱"), ("dog", "🐶"), ("bird", "🐦"), ("fish", "🐟"),
    ("ok", "👍"), ("clap", "👏"), ("wave", "👋"), ("pray", "🙏"),
    ("coffee", "☕"), ("beer", "🍺"), ("pizza", "🍕"), ("apple", "🍎"),
    ("car", "🚗"), ("plane", "✈️"), ("rocket", "🚀"), ("clock", "⏰"),
    ("book", "📖"), ("music", "🎵"), ("gift", "🎁"), ("party", "🎉"),
    ("check", "✅"), ("x", "❌"), ("warning", "⚠️"), ("info", "ℹ️"),
    ("up", "⬆️"), ("down", "⬇️"), ("left", "⬅️"), ("right", "➡️"),
    ("one", "1️⃣"), ("two", "2️⃣"), ("three", "3️⃣"), ("four", "4️⃣"),
    ("five", "5️⃣"), ("ten", "🔟"), ("100", "💯"), ("zzz", "💤"),
];

pub struct EmojiPlugin;

impl Plugin for EmojiPlugin {
    fn name(&self) -> &str { "emoji" }

    fn on_candidate(&mut self, candidates: &mut Vec<Candidate>) {
        let mut extras: Vec<Candidate> = Vec::new();

        for cand in candidates.iter() {
            let lower = cand.text.to_lowercase();
            for &(keyword, emoji) in EMOJI_TABLE {
                if lower.contains(keyword) {
                    extras.push(Candidate {
                        text: format!("{}  {}", emoji, cand.text),
                        comment: format!("🤖 {}", keyword),
                        index: 0,
                        quality: cand.quality + 0.1,
                    });
                    break; // one emoji per candidate
                }
            }
        }

        if !extras.is_empty() {
            // Prepend emoji candidates
            extras.extend(candidates.drain(..));
            *candidates = extras;
        }
    }
}

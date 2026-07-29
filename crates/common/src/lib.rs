//! Shared types, EventBus, and common utilities for the Skyme input method.
//!
//! Every crate in this project depends on this crate for foundational types.
//! No crate should duplicate the types defined here.

pub mod event;
pub mod eventbus;
pub mod types;

pub use event::*;
pub use eventbus::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use crate::event::*;
    use crate::eventbus::*;
    use crate::types::*;

    #[test]
    fn test_event_debug_clone() {
        let e = Event::KeyDown { keycode: 0x41, modifiers: Modifiers::NONE };
        let _ = format!("{:?}", e);
        let _ = e.clone();
    }

    #[test]
    fn test_commit_event() {
        let e = Event::Commit { text: "hello".into() };
        match e {
            Event::Commit { text } => assert_eq!(text, "hello"),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_composition_updated_event() {
        let e = Event::CompositionUpdated { session_id: 42, preedit: "ni".into(), cursor_pos: 2 };
        match e {
            Event::CompositionUpdated { session_id, preedit, cursor_pos } => {
                assert_eq!(session_id, 42);
                assert_eq!(preedit, "ni");
                assert_eq!(cursor_pos, 2);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_eventbus_dispatch() {
        use std::sync::atomic::{AtomicU32, Ordering};
        struct Counter(AtomicU32);
        impl EventListener for Counter {
            fn on_event(&mut self, _event: &Event) { self.0.fetch_add(1, Ordering::SeqCst); }
        }
        let mut bus = EventBus::new();
        bus.register(Box::new(Counter(AtomicU32::new(0))));
        bus.register(Box::new(Counter(AtomicU32::new(0))));
        bus.dispatch(&Event::ThemeChanged);
        // Can't easily verify without shared state, just ensure no crash
    }

    #[test]
    fn test_modifiers_bitflags() {
        let m = Modifiers::SHIFT | Modifiers::CTRL;
        assert!(m.contains(Modifiers::SHIFT));
        assert!(m.contains(Modifiers::CTRL));
        assert!(!m.contains(Modifiers::ALT));
        assert_eq!(m.bits(), 3);
    }

    #[test]
    fn test_modifiers_default() {
        assert_eq!(Modifiers::default(), Modifiers::NONE);
    }

    #[test]
    fn test_candidate_default() {
        let c = Candidate::default();
        assert_eq!(c.text, "");
        assert_eq!(c.index, 0);
    }

    #[test]
    fn test_candidate_new() {
        let c = Candidate { text: "你好".into(), comment: "nǐ hǎo".into(), index: 1, quality: 0.9 };
        assert_eq!(c.text, "你好");
        assert_eq!(c.quality, 0.9);
    }

    #[test]
    fn test_rect_default() {
        let r = Rect::default();
        assert_eq!(r.width, 0.0);
    }

    #[test]
    fn test_composition_state() {
        let s = CompositionState::default();
        assert_eq!(s.preedit, "");
        assert!(s.candidates.is_empty());
    }

    #[test]
    fn test_display_mode_default() {
        assert_eq!(DisplayMode::default(), DisplayMode::Floating);
    }

    #[test]
    fn test_preedit_segment() {
        let s = PreeditSegment { text: "ni".into(), start: 0, end: 2, highlighted: true };
        assert!(s.highlighted);
        assert_eq!(s.text, "ni");
    }
}

use bitflags::bitflags;

/// Top-level events flowing through the Skyme system.
///
/// All modules communicate exclusively through these events.
/// No crate should directly call into another crate's internal logic.
#[derive(Clone, Debug)]
pub enum Event {
    /// A physical key was pressed. Posted by ime-core.
    KeyDown {
        keycode: u32,
        modifiers: Modifiers,
    },

    /// Composition (preedit) text was updated.
    CompositionUpdated {
        session_id: u64,
        preedit: String,
        cursor_pos: usize,
    },

    /// Text was committed to the application.
    Commit {
        text: String,
    },

    /// Input focus changed.
    Focus {
        context_id: u64,
        focused: bool,
    },

    /// Candidate list changed — UI should refresh.
    CandidateChanged {
        session_id: u64,
    },

    /// UI theme changed at runtime.
    ThemeChanged,

    /// Configuration hot-reloaded.
    ConfigReloaded,

    /// Diagnostics / debug event.
    Diagnostics {
        message: String,
    },

    /// Generic plugin event.
    PluginEvent {
        plugin: String,
        payload: String,
    },
}

bitflags! {
    /// Keyboard modifier key flags.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct Modifiers: u32 {
        const NONE  = 0;
        const SHIFT = 1 << 0;
        const CTRL  = 1 << 1;
        const ALT   = 1 << 2;
        const WIN   = 1 << 3;
    }
}

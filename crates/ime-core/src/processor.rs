//! Main TSF text service processor — orchestrates the input method lifecycle.

use crate::com_service::SkymeTextService;
use crate::keyevent::KeyEvent;
use crate::registrar::ClsidRegistrar;
use skyme_common::event::Event;
use skyme_common::eventbus::EventBus;
use skyme_rime_engine::{Engine, RimeResult, Session};

/// The central processor that ties TSF, the Rime engine, and the event bus together.
pub struct TextServiceProcessor {
    engine: Engine,
    event_bus: EventBus,
    tsf_service: SkymeTextService,
    registrar: ClsidRegistrar,
    active_session: Option<Session>,
}

impl TextServiceProcessor {
    pub fn new() -> Self {
        Self {
            engine: Engine::new(),
            event_bus: EventBus::new(),
            tsf_service: SkymeTextService::new(),
            registrar: ClsidRegistrar::new(),
            active_session: None,
        }
    }

    /// Initialise the Rime engine.
    pub fn init_engine(&mut self, shared_dir: &str, user_dir: &str, dist_name: &str) -> RimeResult<()> {
        self.engine.initialize(shared_dir, user_dir, dist_name)
    }

    /// Activate the TSF text service.
    pub fn activate_tsf(&mut self, ptim: *const std::ffi::c_void, tid: u32) -> Result<(), crate::ImeError> {
        self.tsf_service.activate(ptim, tid)
    }

    /// Deactivate the TSF text service.
    pub fn deactivate_tsf(&mut self) {
        self.tsf_service.deactivate();
    }

    /// Register the COM class with Windows.
    pub fn register_com(&self) -> Result<(), String> {
        self.registrar.register()
    }

    /// Handle a key event: process through engine, update composition.
    pub fn handle_key(&mut self, event: &KeyEvent) -> bool {
        let Some(ref session) = self.active_session else {
            // Auto-create session if not yet active
            match self.engine.create_session() {
                Ok(s) => { self.active_session = Some(s); }
                Err(e) => { log::error!("Failed to create session: {}", e); return false; }
            }
            return self.handle_key(event);
        };

        let handled = event.dispatch_to_engine(&self.engine, session);
        if handled {
            self.dispatch_event(Event::CompositionUpdated {
                session_id: session.id(),
                preedit: String::new(), // real data from get_context
                cursor_pos: 0,
            });
        }
        handled
    }

    /// Dispatch an event through the bus.
    pub fn dispatch_event(&mut self, event: Event) {
        self.event_bus.dispatch(&event);
    }

    pub fn engine(&self) -> &Engine { &self.engine }
    pub fn engine_mut(&mut self) -> &mut Engine { &mut self.engine }
    pub fn event_bus(&self) -> &EventBus { &self.event_bus }
    pub fn event_bus_mut(&mut self) -> &mut EventBus { &mut self.event_bus }
    pub fn active_session(&self) -> Option<&Session> { self.active_session.as_ref() }
    pub fn tsf_service(&self) -> &SkymeTextService { &self.tsf_service }
}

impl Default for TextServiceProcessor { fn default() -> Self { Self::new() } }

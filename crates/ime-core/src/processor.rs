//! Main TSF text service processor.

use crate::composition::CompositionManager;
use crate::keyevent::KeyEvent;
use crate::registrar::ClsidRegistrar;
use crate::threadmgr::ThreadManager;
use skyme_common::event::Event;
use skyme_common::eventbus::EventBus;
use skyme_rime_engine::{Engine, RimeResult, Session};

/// The central processor that ties TSF, Rime engine, and event bus together.
pub struct TextServiceProcessor {
    engine: Engine,
    event_bus: EventBus,
    registrar: ClsidRegistrar,
    active_session: Option<Session>,
}

impl TextServiceProcessor {
    pub fn new() -> Self {
        Self {
            engine: Engine::new(),
            event_bus: EventBus::new(),
            registrar: ClsidRegistrar::new(),
            active_session: None,
        }
    }

    pub fn init_engine(&mut self, shared_dir: &str, user_dir: &str, dist_name: &str) -> RimeResult<()> {
        self.engine.initialize(shared_dir, user_dir, dist_name)
    }

    pub fn register_com(&self) -> i32 { self.registrar.register() }

    pub fn handle_key(&mut self, event: &KeyEvent) -> bool {
        let session = match &self.active_session {
            Some(s) => s.clone(),
            None => match self.engine.create_session() {
                Ok(s) => { self.active_session = Some(s); self.active_session.as_ref().unwrap().clone() }
                Err(e) => { log::error!("Failed to create session: {}", e); return false; }
            },
        };
        let handled = event.dispatch_to_engine(&self.engine, &session);
        if handled {
            self.dispatch_event(Event::CompositionUpdated {
                session_id: session.id(), preedit: String::new(), cursor_pos: 0,
            });
        }
        handled
    }

    pub fn dispatch_event(&mut self, event: Event) { self.event_bus.dispatch(&event); }
    pub fn engine(&self) -> &Engine { &self.engine }
    pub fn engine_mut(&mut self) -> &mut Engine { &mut self.engine }
    pub fn event_bus(&self) -> &EventBus { &self.event_bus }
    pub fn event_bus_mut(&mut self) -> &mut EventBus { &mut self.event_bus }
    pub fn active_session(&self) -> Option<&Session> { self.active_session.as_ref() }
}

impl Default for TextServiceProcessor { fn default() -> Self { Self::new() } }

//! Skyme TSF text service — the COM object that Windows TSF loads.
//!
//! This module implements `ITfTextInputProcessor` and related COM interfaces.
//! On non-Windows platforms, it provides a stub that compiles but does nothing.


/// The main Skyme TSF text service COM object.
///
/// This is the entry point that Windows TSF calls into.
/// It implements `ITfTextInputProcessor` and `ITfKeyEventSink`.
#[cfg(target_os = "windows")]
pub struct SkymeTextService {
    thread_mgr: ThreadManager,
    composition: CompositionManager,
    client_id: u32,
    activated: bool,
}

#[cfg(target_os = "windows")]
impl SkymeTextService {
    pub fn new() -> Self {
        Self {
            thread_mgr: ThreadManager::new(),
            composition: CompositionManager::new(),
            client_id: 0,
            activated: false,
        }
    }

    /// `ITfTextInputProcessor::Activate` — called by TSF when the service is loaded.
    pub fn activate(&mut self, ptim: *const std::ffi::c_void, tid: u32) -> Result<(), crate::ImeError> {
        log::info!("Skyme TSF activating (client_id={})", tid);
        // In a real build: ptim is ITfThreadMgr*, call QueryInterface, register sinks.
        self.client_id = tid;
        self.thread_mgr.activate_with(tid)?;
        self.activated = true;
        Ok(())
    }

    /// `ITfTextInputProcessor::Deactivate` — called by TSF when the service is unloaded.
    pub fn deactivate(&mut self) {
        if self.activated {
            log::info!("Skyme TSF deactivating");
            self.thread_mgr.deactivate();
            self.activated = false;
        }
    }

    /// Handle a key down event. Called by the key event sink.
    pub fn on_key_down(&mut self, keycode: u32) -> bool {
        let event = KeyEvent::new(keycode, Default::default(), true);
        // TODO: Dispatch to rime-engine via event bus.
        log::debug!("Key down: {}", keycode);
        true
    }

    pub fn is_activated(&self) -> bool { self.activated }
    pub fn client_id(&self) -> u32 { self.client_id }
    pub fn thread_mgr(&self) -> &ThreadManager { &self.thread_mgr }
    pub fn thread_mgr_mut(&mut self) -> &mut ThreadManager { &mut self.thread_mgr }
}

#[cfg(not(target_os = "windows"))]
pub struct SkymeTextService;

#[cfg(not(target_os = "windows"))]
impl SkymeTextService {
    pub fn new() -> Self { Self }
    pub fn activate(&mut self, _ptim: *const std::ffi::c_void, _tid: u32) -> Result<(), crate::ImeError> {
        log::info!("Skyme TSF stub: activate (non-Windows)");
        Ok(())
    }
    pub fn deactivate(&mut self) { log::info!("Skyme TSF stub: deactivate"); }
    pub fn on_key_down(&mut self, keycode: u32) -> bool { log::debug!("Key down (stub): {}", keycode); true }
    pub fn is_activated(&self) -> bool { true }
    pub fn client_id(&self) -> u32 { 0 }
}

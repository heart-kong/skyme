//! Skyme TSF text service — the COM object that Windows TSF loads.

use crate::composition::CompositionManager;
use crate::keyevent::KeyEvent;
use crate::threadmgr::ThreadManager;

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
        Self { thread_mgr: ThreadManager::new(), composition: CompositionManager::new(), client_id: 0, activated: false }
    }

    pub fn activate(&mut self, _ptim: *const std::ffi::c_void, tid: u32) -> Result<(), crate::ImeError> {
        log::info!("Skyme TSF activating (client_id={})", tid);
        self.client_id = tid;
        self.thread_mgr.activate_with(tid).map_err(|e| crate::ImeError::ActivationFailed(e))?;
        self.activated = true;
        Ok(())
    }

    pub fn deactivate(&mut self) {
        if self.activated {
            log::info!("Skyme TSF deactivating");
            self.thread_mgr.deactivate();
            self.activated = false;
        }
    }

    pub fn on_key_down(&mut self, keycode: u32) -> bool {
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
    pub fn thread_mgr(&self) -> &ThreadManager { unimplemented!() }
    pub fn thread_mgr_mut(&mut self) -> &mut ThreadManager { unimplemented!() }
}

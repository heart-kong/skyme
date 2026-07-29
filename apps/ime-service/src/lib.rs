//! Skyme IME Service — the Windows TSF text service COM DLL.
//!
//! COM exports: DllMain, DllRegisterServer, DllUnregisterServer,
//! DllGetClassObject, DllCanUnloadNow.

use skyme_rime_engine::{Engine, RimeResult};
use std::sync::Mutex;

static SERVICE: Mutex<Option<ImeService>> = Mutex::new(None);

pub struct ImeService {
    #[allow(dead_code)]
    engine: Engine,
}

impl ImeService {
    pub fn new() -> Self { Self { engine: Engine::new() } }
    pub fn initialize(shared_dir: &str, user_dir: &str, dist_name: &str) -> RimeResult<()> {
        let mut engine = Engine::new();
        engine.initialize(shared_dir, user_dir, dist_name)?;
        *SERVICE.lock().unwrap() = Some(ImeService { engine });
        log::info!("Skyme IME service initialised");
        Ok(())
    }
    pub fn shutdown() { *SERVICE.lock().unwrap() = None; log::info!("Skyme IME service shut down"); }
}

// ── COM exports ────────────────────────────────────────────────────────────

#[no_mangle] pub extern "system" fn DllMain() -> bool { true }

#[no_mangle]
pub extern "system" fn DllRegisterServer() -> i32 {
    skyme_ime_core::ClsidRegistrar::new().register()
}

#[no_mangle]
pub extern "system" fn DllUnregisterServer() -> i32 {
    skyme_ime_core::ClsidRegistrar::new().unregister()
}

#[no_mangle]
pub extern "system" fn DllGetClassObject(
    clsid: *const skyme_ime_core::com::GUID,
    iid: *const skyme_ime_core::com::GUID,
    ppv: *mut *mut std::ffi::c_void,
) -> i32 {
    skyme_ime_core::com::class_factory::class_factory(clsid, iid, ppv)
}

#[no_mangle]
pub extern "system" fn DllCanUnloadNow() -> i32 { 1 } // S_FALSE

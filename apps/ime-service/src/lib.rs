//! Skyme IME Service — the Windows TSF text service COM DLL.

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

    pub fn shutdown() {
        *SERVICE.lock().unwrap() = None;
        log::info!("Skyme IME service shut down");
    }
}

#[no_mangle]
pub extern "system" fn DllMain() -> bool { true }

#[no_mangle]
pub extern "system" fn DllRegisterServer() -> i32 {
    match skyme_ime_core::ClsidRegistrar::new().register() {
        Ok(_) => { log::info!("DllRegisterServer succeeded"); 0 }
        Err(e) => { log::error!("DllRegisterServer failed: {}", e); 1 }
    }
}

#[no_mangle]
pub extern "system" fn DllUnregisterServer() -> i32 {
    match skyme_ime_core::ClsidRegistrar::new().unregister() {
        Ok(_) => { log::info!("DllUnregisterServer succeeded"); 0 }
        Err(e) => { log::error!("DllUnregisterServer failed: {}", e); 1 }
    }
}

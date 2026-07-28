//! COM registration and unregistration for the Skyme TSF text service.
//!
//! Handles DllRegisterServer / DllUnregisterServer logic.

/// Handles CLSID and profile registration with TSF.
pub struct ClsidRegistrar;

impl ClsidRegistrar {
    pub fn new() -> Self { Self }

    /// Register the text service with Windows TSF.
    /// This would call `ITfCategoryMgr::RegisterCategory` and
    /// `ITfInputProcessorProfileMgr::RegisterProfile`.
    #[cfg(target_os = "windows")]
    pub fn register(&self) -> Result<(), String> {
        log::info!("Registering Skyme TSF text service");
        // TODO: Real COM registration:
        // 1. Register CLSID in registry
        // 2. ITfCategoryMgr::RegisterCategory for GUID_TFCAT_TIP_KEYBOARD
        // 3. ITfInputProcessorProfileMgr::RegisterProfile
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    pub fn register(&self) -> Result<(), String> {
        log::info!("Skyme TSF registration: stub (non-Windows)");
        Ok(())
    }

    /// Unregister the text service.
    #[cfg(target_os = "windows")]
    pub fn unregister(&self) -> Result<(), String> {
        log::info!("Unregistering Skyme TSF text service");
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    pub fn unregister(&self) -> Result<(), String> {
        Ok(())
    }
}

impl Default for ClsidRegistrar { fn default() -> Self { Self::new() } }

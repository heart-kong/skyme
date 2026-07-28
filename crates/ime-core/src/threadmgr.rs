//! TSF thread manager lifecycle management.

/// Manages the TSF thread manager lifecycle.
///
/// Responsible for activating/deactivating the text service in
/// the TSF thread manager, and managing the client ID.
pub struct ThreadManager {
    activated: bool,
    client_id: u32,
}

impl ThreadManager {
    pub fn new() -> Self { Self { activated: false, client_id: 0 } }

    /// Activate with a given TSF client ID.
    pub fn activate_with(&mut self, client_id: u32) -> Result<(), String> {
        self.client_id = client_id;
        self.activated = true;
        log::info!("TSF thread manager activated (client_id={})", client_id);
        Ok(())
    }

    pub fn activate(&mut self) -> Result<(), String> {
        self.activated = true;
        log::info!("TSF thread manager activated");
        Ok(())
    }

    pub fn deactivate(&mut self) {
        if self.activated {
            self.activated = false;
            log::info!("TSF thread manager deactivated");
        }
    }

    pub fn client_id(&self) -> u32 { self.client_id }
    pub fn is_activated(&self) -> bool { self.activated }
}

impl Default for ThreadManager { fn default() -> Self { Self::new() } }

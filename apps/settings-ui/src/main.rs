//! Skyme Settings UI — standalone configuration application.
//!
//! Native Win32 dialog for configuring the input method.

use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(target_os = "windows")]
mod dialog;

fn main() {
    env_logger::init();
    log::info!("Skyme Settings UI starting");

    #[cfg(target_os = "windows")]
    dialog::run_settings_dialog();

    #[cfg(not(target_os = "windows"))]
    log::info!("Settings UI requires Windows");
}

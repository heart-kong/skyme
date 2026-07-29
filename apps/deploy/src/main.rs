//! Skyme Deploy — schema deployment tool.
//!
//! Initialises the Rime engine and runs deployment (schema compilation).
//! Usage: RIME_SHARED_DATA_DIR=/usr/share/rime-data skyme-deploy

fn main() {
    env_logger::init();
    log::info!("Skyme Deploy starting");

    let shared = std::env::var("RIME_SHARED_DATA_DIR").unwrap_or_else(|_| "/usr/share/rime-data".into());
    let user = std::env::var("RIME_USER_DATA_DIR").unwrap_or_else(|_| {
        let h = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        format!("{}/.local/share/rime", h)
    });

    let mut engine = skyme_rime_engine::Engine::new();
    match engine.initialize(&shared, &user, "Skyme") {
        Ok(_) => {
            log::info!("Engine initialized, deployment would run next");
            // TODO: Access engine's RimeApi and pass to Deployer::deploy()
        }
        Err(e) => log::warn!("Engine init (expected in dev): {}", e),
    }

    log::info!("Skyme Deploy completed");
}

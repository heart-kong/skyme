/// Skyme IME Service binary entry point (debug/test harness).
///
/// The real IME service is loaded by TSF as a COM DLL (lib.rs / cdylib).
/// This binary exists for testing. The library is accessible via the
/// `skyme_ime_service` crate name in cargo test.
fn main() {
    env_logger::init();
    log::info!("Skyme IME Service (test harness) starting");

    // The library is compiled separately — main.rs is just a placeholder.
    // Run `cargo test -p skyme-ime-service` to test the library.

    log::info!("Skyme IME Service shutting down");
}

//! Rime schema deployment.
//!
//! Compiles `.yaml` schema files into `.bin` files that librime can load.
//! This is a blocking operation that may take several seconds.

use crate::error::{RimeError, RimeResult};
use crate::ffi::RimeTraits;
use crate::raw;
use std::ffi::CString;
use std::time::{Duration, Instant};

/// Handles Rime deployment — schema compilation and maintenance.
pub struct Deployer {
    running: bool,
}

impl Deployer {
    pub fn new() -> Self {
        Self { running: false }
    }

    /// Initialise the deployer with data directories, then start maintenance.
    ///
    /// This is a synchronous call that blocks until deployment finishes.
    /// Progress updates are periodically logged.
    ///
    /// # Arguments
    ///
    /// * `shared_data_dir` — Path to Rime's shared data.
    /// * `user_data_dir` — Path to user-specific data.
    /// * `distribution_name` — Frontend name (e.g. "Skyme").
    /// * `full_check` — If true, recompile all schemas regardless of timestamps.
    pub fn deploy(
        &mut self,
        shared_data_dir: &str,
        user_data_dir: &str,
        distribution_name: &str,
        full_check: bool,
    ) -> RimeResult<()> {
        if self.running {
            return Err(RimeError::DeployFailed(
                "Deployment already in progress".into(),
            ));
        }

        let shared_c = CString::new(shared_data_dir)?;
        let user_c = CString::new(user_data_dir)?;
        let dist_c = CString::new(distribution_name)?;

        let traits = RimeTraits::new(
            shared_c.as_ptr(),
            user_c.as_ptr(),
            dist_c.as_ptr(),
        );

        self.running = true;

        unsafe {
            if !raw::deployer_initialize(&traits) {
                self.running = false;
                return Err(RimeError::DeployFailed(
                    "RimeDeployer_Initialize failed".into(),
                ));
            }

            log::info!(
                "Rime deployment starting (full_check={})",
                full_check
            );
            raw::deployer_start_maintenance(full_check);

            // Poll until maintenance completes.
            let start = Instant::now();
            while raw::deployer_is_maintenance_running() {
                std::thread::sleep(Duration::from_millis(200));
                let elapsed = start.elapsed();
                if elapsed.as_secs() > 0 && elapsed.as_secs() % 5 == 0 {
                    log::info!("Rime deployment in progress... ({:.0}s)", elapsed.as_secs());
                }
            }

            raw::deployer_join_maintenance_thread();

            let elapsed = start.elapsed();
            log::info!("Rime deployment completed in {:.1}s", elapsed.as_secs_f64());
        }

        self.running = false;
        Ok(())
    }

    /// Check if a deployment is currently running.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Cancel an ongoing deployment (stub — librime does not support cancellation).
    pub fn cancel(&mut self) {
        log::warn!("Rime deployment cancellation not supported by librime");
    }
}

impl Default for Deployer {
    fn default() -> Self {
        Self::new()
    }
}

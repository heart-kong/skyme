use crate::error::{RimeError, RimeResult};
use crate::ffi::{RimeApi, RimeTraits};
use crate::raw;
use std::ffi::CString;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Handles Rime deployment — schema compilation and maintenance.
pub struct Deployer { running: bool }

impl Deployer {
    pub fn new() -> Self { Self { running: false } }

    pub fn deploy(
        &mut self,
        api: &RimeApi,
        shared_data_dir: &str,
        user_data_dir: &str,
        distribution_name: &str,
        full_check: bool,
    ) -> RimeResult<()> {
        if self.running { return Err(RimeError::DeployFailed("Deployment already in progress".into())); }

        let shared_c = CString::new(shared_data_dir)?;
        let user_c = CString::new(user_data_dir)?;
        let dist_c = CString::new(distribution_name)?;
        let traits = RimeTraits::new(shared_c.as_ptr(), user_c.as_ptr(), dist_c.as_ptr());

        self.running = true;
        unsafe {
            if !raw::deployer_initialize(api, &traits) {
                self.running = false;
                return Err(RimeError::DeployFailed("RimeDeployer_Initialize failed".into()));
            }
            log::info!("Rime deployment starting (full_check={})", full_check);
            raw::deployer_start_maintenance(api, full_check);

            let start = Instant::now();
            while raw::deployer_is_maintenance_running(api) {
                std::thread::sleep(Duration::from_millis(200));
            }
            raw::deployer_join_maintenance_thread(api);
            log::info!("Rime deployment completed in {:.1}s", start.elapsed().as_secs_f64());
        }
        self.running = false;
        Ok(())
    }

    pub fn is_running(&self) -> bool { self.running }
    pub fn cancel(&mut self) { log::warn!("Cancellation not supported"); }
}

impl Default for Deployer { fn default() -> Self { Self::new() } }

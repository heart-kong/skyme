//! Unsafe, low-level wrappers around librime function pointers.
//!
//! Every function takes a `&RimeApi` as the first argument, which is the
//! runtime-loaded set of function pointers. This replaces the old `extern "C"` block.

use crate::error::RimeResult;
use crate::ffi::{RimeApi, RimeCandidateList, RimeCommit, RimeContext, RimePreedit};
use crate::ffi::{RimeSchema, RimeSchemaList, RimeStatus, RimeTraits};
use std::ffi::c_char;
use std::ffi::{CStr, CString};

// ── lifecycle ──

pub unsafe fn setup_logging(api: &RimeApi) { (api.RimeSetupLogging)() }
pub unsafe fn initialize(api: &RimeApi, traits: &RimeTraits) -> bool { (api.RimeInitialize)(traits as *const RimeTraits) }
pub unsafe fn finalize(api: &RimeApi) { (api.RimeFinalize)() }

// ── sessions ──

pub unsafe fn create_session(api: &RimeApi) -> Option<u64> {
    let id = (api.RimeCreateSession)();
    if id == 0 { None } else { Some(id) }
}
pub unsafe fn destroy_session(api: &RimeApi, id: u64) { (api.RimeDestroySession)(id) }
pub unsafe fn find_session(api: &RimeApi, id: u64) -> bool { (api.RimeFindSession)(id) }

// ── key processing ──

pub unsafe fn process_key(api: &RimeApi, session_id: u64, keycode: i32, modifiers: i32) -> bool {
    (api.RimeProcessKey)(session_id, keycode, modifiers)
}
pub unsafe fn commit_composition(api: &RimeApi, session_id: u64) { (api.RimeCommitComposition)(session_id) }
pub unsafe fn clear_composition(api: &RimeApi, session_id: u64) { (api.RimeClearComposition)(session_id) }

// ── context ──

pub unsafe fn get_context(api: &RimeApi, session_id: u64) -> RimeResult<Option<RimeContext>> {
    let mut ctx = RimeContext::default();
    if !(api.RimeGetContext)(session_id, &mut ctx) { return Ok(None); }
    Ok(Some(ctx))
}
pub unsafe fn free_context(api: &RimeApi, ctx: &mut RimeContext) { (api.RimeFreeContext)(ctx) }

// ── commit ──

pub unsafe fn get_commit(api: &RimeApi, session_id: u64) -> RimeResult<Option<RimeCommit>> {
    let mut commit = RimeCommit::default();
    if !(api.RimeGetCommit)(session_id, &mut commit) { return Ok(None); }
    Ok(Some(commit))
}
pub unsafe fn free_commit(api: &RimeApi, commit: &mut RimeCommit) { (api.RimeFreeCommit)(commit) }

// ── status ──

pub unsafe fn get_status(api: &RimeApi, session_id: u64) -> RimeResult<Option<RimeStatus>> {
    let mut status = RimeStatus::default();
    if !(api.RimeGetStatus)(session_id, &mut status) { return Ok(None); }
    Ok(Some(status))
}
pub unsafe fn free_status(api: &RimeApi, status: &mut RimeStatus) { (api.RimeFreeStatus)(status) }

// ── candidates ──

pub unsafe fn select_candidate(api: &RimeApi, session_id: u64, index: i32) -> bool {
    (api.RimeSelectCandidate)(session_id, index)
}
pub unsafe fn free_candidate_list(api: &RimeApi, list: &mut RimeCandidateList) {
    (api.RimeFreeCandidateList)(list)
}

// ── options ──

pub unsafe fn set_option(api: &RimeApi, session_id: u64, option: &str, value: bool) -> RimeResult<bool> {
    let c = CString::new(option)?;
    Ok((api.RimeSetOption)(session_id, c.as_ptr(), value))
}
pub unsafe fn get_option(api: &RimeApi, session_id: u64, option: &str) -> RimeResult<bool> {
    let c = CString::new(option)?;
    Ok((api.RimeGetOption)(session_id, c.as_ptr()))
}

// ── schema ──

pub unsafe fn get_schema_list(api: &RimeApi) -> RimeResult<Option<RimeSchemaList>> {
    let mut list = RimeSchemaList::default();
    if !(api.RimeGetSchemaList)(&mut list) { return Ok(None); }
    Ok(Some(list))
}
pub unsafe fn free_schema_list(api: &RimeApi, list: &mut RimeSchemaList) { (api.RimeFreeSchemaList)(list) }
pub unsafe fn select_schema(api: &RimeApi, session_id: u64, schema_id: &str) -> RimeResult<bool> {
    let c = CString::new(schema_id)?;
    Ok((api.RimeSelectSchema)(session_id, c.as_ptr()))
}
pub unsafe fn current_schema(api: &RimeApi, session_id: u64) -> RimeResult<Option<RimeSchema>> {
    let mut schema = RimeSchema::default();
    if !(api.RimeCurrentSchema)(session_id, &mut schema) { return Ok(None); }
    Ok(Some(schema))
}

// ── deployer ──

pub unsafe fn deployer_initialize(api: &RimeApi, traits: &RimeTraits) -> bool {
    (api.RimeDeployer_Initialize)(traits as *const RimeTraits)
}
pub unsafe fn deployer_start_maintenance(api: &RimeApi, full_check: bool) { (api.RimeDeployer_StartMaintenance)(full_check) }
pub unsafe fn deployer_is_maintenance_running(api: &RimeApi) -> bool { (api.RimeDeployer_IsMaintenanceRunning)() }
pub unsafe fn deployer_join_maintenance_thread(api: &RimeApi) { (api.RimeDeployer_JoinMaintenanceThread)() }

// ── C string helpers ──

pub unsafe fn cstr_to_opt<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr.is_null() { None } else { CStr::from_ptr(ptr).to_str().ok() }
}
pub unsafe fn cstr_to_owned(ptr: *mut c_char) -> Option<String> {
    if ptr.is_null() { None } else { CStr::from_ptr(ptr).to_str().ok().map(String::from) }
}

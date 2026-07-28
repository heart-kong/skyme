//! Unsafe, low-level wrappers around `extern "C"` FFI calls.
//!
//! This module bridges raw C pointers and Rust-friendly types.
//! Functions here are `unsafe` — they assume librime is initialized and
//! the provided pointers/session-ids are valid.
//!
//! # Safety contract
//!
//! Callers must guarantee:
//! - `RimeInitialize()` has been called and returned true.
//! - Session IDs come from `RimeCreateSession()`.
//! - Pointers are non-null where the docs say so.
//! - `RimeFinalize()` has NOT been called yet.

use crate::error::RimeResult;
use crate::ffi::*;
use std::ffi::{CStr, CString};

// ── lifecycle ───────────────────────────────────────────────────────────────

pub unsafe fn setup_logging() {
    unsafe { RimeSetupLogging() }
}

pub unsafe fn initialize(traits: &RimeTraits) -> bool {
    unsafe { RimeInitialize(traits as *const RimeTraits) }
}

pub unsafe fn finalize() {
    unsafe { RimeFinalize() }
}

// ── sessions ────────────────────────────────────────────────────────────────

pub unsafe fn create_session() -> Option<u64> {
    let id = unsafe { RimeCreateSession() };
    if id == 0 { None } else { Some(id) }
}

pub unsafe fn destroy_session(id: u64) {
    unsafe { RimeDestroySession(id) }
}

pub unsafe fn find_session(id: u64) -> bool {
    unsafe { RimeFindSession(id) }
}

// ── key processing ──────────────────────────────────────────────────────────

pub unsafe fn process_key(session_id: u64, keycode: i32, modifiers: i32) -> bool {
    unsafe { RimeProcessKey(session_id, keycode, modifiers) }
}

pub unsafe fn commit_composition(session_id: u64) {
    unsafe { RimeCommitComposition(session_id) }
}

pub unsafe fn clear_composition(session_id: u64) {
    unsafe { RimeClearComposition(session_id) }
}

// ── context ─────────────────────────────────────────────────────────────────

/// Fetch the full context for a session. Returns `None` if there is no context.
pub unsafe fn get_context(session_id: u64) -> RimeResult<Option<RimeContext>> {
    let mut ctx = RimeContext::default();
    let ok = unsafe { RimeGetContext(session_id, &mut ctx) };
    if !ok {
        return Ok(None);
    }
    Ok(Some(ctx))
}

/// Free a context allocated by librime.
pub unsafe fn free_context(ctx: &mut RimeContext) {
    unsafe { RimeFreeContext(ctx) }
}

// ── commit ──────────────────────────────────────────────────────────────────

/// Get the latest committed text. Returns `None` if nothing has been committed.
pub unsafe fn get_commit(session_id: u64) -> RimeResult<Option<RimeCommit>> {
    let mut commit = RimeCommit::default();
    let ok = unsafe { RimeGetCommit(session_id, &mut commit) };
    if !ok {
        return Ok(None);
    }
    Ok(Some(commit))
}

pub unsafe fn free_commit(commit: &mut RimeCommit) {
    unsafe { RimeFreeCommit(commit) }
}

// ── status ──────────────────────────────────────────────────────────────────

pub unsafe fn get_status(session_id: u64) -> RimeResult<Option<RimeStatus>> {
    let mut status = RimeStatus::default();
    let ok = unsafe { RimeGetStatus(session_id, &mut status) };
    if !ok {
        return Ok(None);
    }
    Ok(Some(status))
}

pub unsafe fn free_status(status: &mut RimeStatus) {
    unsafe { RimeFreeStatus(status) }
}

// ── candidates ──────────────────────────────────────────────────────────────

pub unsafe fn select_candidate(session_id: u64, index: i32) -> bool {
    unsafe { RimeSelectCandidate(session_id, index) }
}

pub unsafe fn free_candidate_list(list: &mut RimeCandidateList) {
    unsafe { RimeFreeCandidateList(list) }
}

// ── options ─────────────────────────────────────────────────────────────────

pub unsafe fn set_option(session_id: u64, option: &str, value: bool) -> RimeResult<bool> {
    let c = CString::new(option)?;
    Ok(unsafe { RimeSetOption(session_id, c.as_ptr(), value) })
}

pub unsafe fn get_option(session_id: u64, option: &str) -> RimeResult<bool> {
    let c = CString::new(option)?;
    Ok(unsafe { RimeGetOption(session_id, c.as_ptr()) })
}

// ── schema ──────────────────────────────────────────────────────────────────

pub unsafe fn get_schema_list() -> RimeResult<Option<RimeSchemaList>> {
    let mut list = RimeSchemaList::default();
    let ok = unsafe { RimeGetSchemaList(&mut list) };
    if !ok {
        return Ok(None);
    }
    Ok(Some(list))
}

pub unsafe fn free_schema_list(list: &mut RimeSchemaList) {
    unsafe { RimeFreeSchemaList(list) }
}

pub unsafe fn select_schema(session_id: u64, schema_id: &str) -> RimeResult<bool> {
    let c = CString::new(schema_id)?;
    Ok(unsafe { RimeSelectSchema(session_id, c.as_ptr()) })
}

pub unsafe fn current_schema(session_id: u64) -> RimeResult<Option<RimeSchema>> {
    let mut schema = RimeSchema::default();
    let ok = unsafe { RimeCurrentSchema(session_id, &mut schema) };
    if !ok {
        return Ok(None);
    }
    Ok(Some(schema))
}

// ── deployer ─────────────────────────────────────────────────────────────────

pub unsafe fn deployer_initialize(traits: &RimeTraits) -> bool {
    unsafe { RimeDeployer_Initialize(traits as *const RimeTraits) }
}

pub unsafe fn deployer_start_maintenance(full_check: bool) {
    unsafe { RimeDeployer_StartMaintenance(full_check) }
}

pub unsafe fn deployer_is_maintenance_running() -> bool {
    unsafe { RimeDeployer_IsMaintenanceRunning() }
}

pub unsafe fn deployer_join_maintenance_thread() {
    unsafe { RimeDeployer_JoinMaintenanceThread() }
}

// ── C string helpers ────────────────────────────────────────────────────────

/// Convert a `*const c_char` to `&str` (librime-owned memory, no copy).
/// Returns `None` for null pointers or invalid UTF-8.
pub unsafe fn cstr_to_opt<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(ptr).to_str().ok() }
}

/// Convert `*mut c_char` to an owned `String`, then give ownership back.
/// Used for fields we want to copy out before freeing the parent struct.
pub unsafe fn cstr_to_owned(ptr: *mut c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(ptr).to_str().ok().map(String::from) }
}

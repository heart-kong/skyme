//! Safe Rust API for interacting with a librime engine instance.
//!
//! Uses runtime dynamic loading (`libloading`) — no compile-time linking needed.
//! librime (`rime.dll` / `librime.so` / `librime.dylib`) must be available at runtime
//! in the library search path.

use crate::candidate::CandidateList;
use crate::error::{RimeError, RimeResult};
use crate::ffi::{RimeApi, RimeTraits};
use crate::raw;
use crate::schema::SchemaInfo;
use crate::session::Session;
use skyme_common::{Candidate, Modifiers};
use std::ffi::CString;
use std::sync::Arc;

/// Whether a keypress was consumed by the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyProcessResult { Handled, Passthrough }

/// High-level safe API for interacting with a Rime engine instance.
///
/// librime is loaded dynamically via `libloading`. The library handle
/// is kept alive for the lifetime of the `Engine`.
pub struct Engine {
    api: Option<Arc<RimeApi>>,
}

impl Engine {
    pub fn new() -> Self { Self { api: None } }

    /// Load librime and initialise the engine.
    ///
    /// Searches for `rime.dll` / `librime.so` / `librime.dylib` in the library path
    /// or in the directory specified by `rime_lib_dir`.
    pub fn initialize(
        &mut self,
        shared_data_dir: &str,
        user_data_dir: &str,
        distribution_name: &str,
    ) -> RimeResult<()> {
        if self.api.is_some() {
            log::warn!("Rime engine already initialized");
            return Ok(());
        }

        let lib = unsafe { libloading::Library::new("rime") }
            .or_else(|_| unsafe { libloading::Library::new("librime") })
            .or_else(|_| {
                // Try with platform-specific names
                #[cfg(target_os = "windows")]
                { unsafe { libloading::Library::new("rime.dll") } }
                #[cfg(not(target_os = "windows"))]
                { unsafe { libloading::Library::new("librime.so.1") } }
                    .or_else(|_| unsafe { libloading::Library::new("librime.dylib") })
            })
            .map_err(|e| RimeError::LibraryLoadFailed(e.to_string()))?;

        let api = Arc::new(unsafe { RimeApi::new(Arc::new(lib)) }
            .map_err(|e| RimeError::LibraryLoadFailed(e.to_string()))?);

        let shared_c = CString::new(shared_data_dir)?;
        let user_c = CString::new(user_data_dir)?;
        let dist_c = CString::new(distribution_name)?;
        let traits = RimeTraits::new(shared_c.as_ptr(), user_c.as_ptr(), dist_c.as_ptr());

        unsafe { raw::setup_logging(&api); }
        if !unsafe { raw::initialize(&api, &traits) } {
            return Err(RimeError::InitializeFailed);
        }

        self.api = Some(api);
        log::info!("Rime engine initialized (shared={}, user={})", shared_data_dir, user_data_dir);
        Ok(())
    }

    pub fn finalize(&mut self) {
        if let Some(ref api) = self.api {
            unsafe { raw::finalize(api); }
            self.api = None;
            log::info!("Rime engine finalized");
        }
    }

    pub fn is_initialized(&self) -> bool { self.api.is_some() }
    pub fn rime_process_key_fn(&self) -> Option<usize> {
        self.api.as_ref().map(|api| api.RimeProcessKey as usize)
    }
    pub fn rime_get_commit_fn(&self) -> Option<usize> {
        self.api.as_ref().map(|api| api.RimeGetCommit as usize)
    }
    pub fn rime_free_commit_fn(&self) -> Option<usize> {
        self.api.as_ref().map(|api| api.RimeFreeCommit as usize)
    }

    fn api(&self) -> RimeResult<&RimeApi> {
        self.api.as_deref().ok_or(RimeError::NotInitialized)
    }

    // ── session ──

    pub fn create_session(&self) -> RimeResult<Session> {
        let api = self.api()?;
        unsafe { raw::create_session(api).map(Session::new).ok_or(RimeError::SessionFailed(0)) }
    }

    pub fn find_session(&self, session: &Session) -> bool {
        self.api().map(|api| unsafe { raw::find_session(api, session.id()) }).unwrap_or(false)
    }

    // ── key processing ──

    pub fn process_key(&self, session: &Session, keycode: u32, modifiers: Modifiers) -> KeyProcessResult {
        let ok = self.api().map(|api| unsafe { raw::process_key(api, session.id(), keycode as i32, modifiers.bits() as i32) }).unwrap_or(false);
        if ok { KeyProcessResult::Handled } else { KeyProcessResult::Passthrough }
    }

    pub fn commit_composition(&self, session: &Session) {
        if let Ok(api) = self.api() { unsafe { raw::commit_composition(api, session.id()) } }
    }

    pub fn clear_composition(&self, session: &Session) {
        if let Ok(api) = self.api() { unsafe { raw::clear_composition(api, session.id()) } }
    }

    // ── context ──

    pub fn get_context(&self, session: &Session) -> RimeResult<Option<SessionContext>> {
        let api = self.api()?;
        unsafe {
            let mut ctx = match raw::get_context(api, session.id())? {
                Some(c) => c, None => return Ok(None),
            };
            let preedit = raw::cstr_to_owned(ctx.composition.preedit);
            let candidates = Self::extract_candidates(&ctx.composition.cand);
            let commit_preview = raw::cstr_to_owned(ctx.commit_text_preview);
            raw::free_context(api, &mut ctx);
            Ok(Some(SessionContext {
                preedit, cursor_pos: ctx.composition.cursor_pos as usize,
                select_labels: ctx.select_label_count, commit_text_preview: commit_preview, candidates,
            }))
        }
    }

    pub fn get_commit(&self, session: &Session) -> RimeResult<Option<CommitText>> {
        let api = self.api()?;
        unsafe {
            let mut commit = match raw::get_commit(api, session.id())? { Some(c) => c, None => return Ok(None) };
            let text = raw::cstr_to_owned(commit.text).unwrap_or_default();
            raw::free_commit(api, &mut commit);
            Ok(Some(CommitText { text }))
        }
    }

    pub fn get_status(&self, session: &Session) -> RimeResult<Option<SessionStatus>> {
        let api = self.api()?;
        unsafe {
            let mut status = match raw::get_status(api, session.id())? { Some(s) => s, None => return Ok(None) };
            let schema_id = raw::cstr_to_owned(status.schema_id).unwrap_or_default();
            let schema_name = raw::cstr_to_owned(status.schema_name).unwrap_or_default();
            raw::free_status(api, &mut status);
            Ok(Some(SessionStatus {
                schema_id, schema_name,
                is_composing: status.is_composing, is_ascii_mode: status.is_ascii_mode,
                is_full_shape: status.is_full_shape, is_simplified: status.is_simplified,
                is_disabled: status.is_disabled,
            }))
        }
    }

    // ── candidates ──

    pub fn select_candidate(&self, session: &Session, index: u32) -> bool {
        self.api().map(|api| unsafe { raw::select_candidate(api, session.id(), index as i32) }).unwrap_or(false)
    }

    pub fn get_candidates(&self, session: &Session) -> RimeResult<Option<CandidateList>> {
        let api = self.api()?;
        unsafe {
            let mut ctx = match raw::get_context(api, session.id())? { Some(c) => c, None => return Ok(None) };
            let list = Self::extract_candidates(&ctx.composition.cand);
            let (page_no, page_size, is_last) = (ctx.composition.cand.page_no as u32, ctx.composition.cand.page_size as u32, ctx.composition.cand.is_last_page);
            raw::free_context(api, &mut ctx);
            Ok(Some(CandidateList::new(list, page_no, page_size, is_last)))
        }
    }

    fn extract_candidates(raw_list: &crate::ffi::RimeCandidateList) -> Vec<Candidate> {
        if raw_list.candidates.is_null() || raw_list.length <= 0 { return Vec::new(); }
        let mut out = Vec::with_capacity(raw_list.length as usize);
        for i in 0..raw_list.length as isize {
            unsafe {
                let cand = &*raw_list.candidates.offset(i);
                out.push(Candidate {
                    text: raw::cstr_to_owned(cand.text).unwrap_or_default(),
                    comment: raw::cstr_to_owned(cand.comment).unwrap_or_default(),
                    index: (raw_list.candidate_index + i as i32) as u32,
                    quality: 0.0,
                });
            }
        }
        out
    }

    // ── options ──

    pub fn set_option(&self, session: &Session, option: &str, value: bool) -> RimeResult<bool> {
        unsafe { raw::set_option(self.api()?, session.id(), option, value) }
    }
    pub fn get_option(&self, session: &Session, option: &str) -> RimeResult<bool> {
        unsafe { raw::get_option(self.api()?, session.id(), option) }
    }

    // ── schema ──

    pub fn get_schema_list(&self) -> RimeResult<Vec<SchemaInfo>> {
        let api = self.api()?;
        unsafe {
            let mut list = match raw::get_schema_list(api)? { Some(l) => l, None => return Ok(Vec::new()) };
            let mut out = Vec::with_capacity(list.length as usize);
            for i in 0..list.length as isize {
                let item = &*list.schemas.offset(i);
                out.push(SchemaInfo::new(
                    raw::cstr_to_owned(item.schema.schema_id).unwrap_or_default(),
                    raw::cstr_to_owned(item.schema.name).unwrap_or_default(),
                ));
            }
            raw::free_schema_list(api, &mut list);
            Ok(out)
        }
    }

    pub fn select_schema(&self, session: &Session, schema_id: &str) -> RimeResult<bool> {
        unsafe { raw::select_schema(self.api()?, session.id(), schema_id) }
    }

    pub fn current_schema(&self, session: &Session) -> RimeResult<Option<String>> {
        let api = self.api()?;
        unsafe {
            let s = match raw::current_schema(api, session.id())? { Some(s) => s, None => return Ok(None) };
            Ok(Some(raw::cstr_to_owned(s.name).unwrap_or_default()))
        }
    }
}

impl Default for Engine { fn default() -> Self { Self::new() } }
impl Drop for Engine { fn drop(&mut self) { self.finalize(); } }

// ── Data types ──

#[derive(Debug, Clone)]
pub struct SessionContext {
    pub preedit: Option<String>,
    pub cursor_pos: usize,
    pub select_labels: i32,
    pub commit_text_preview: Option<String>,
    pub candidates: Vec<Candidate>,
}

#[derive(Debug, Clone)]
pub struct CommitText { pub text: String }

#[derive(Debug, Clone)]
pub struct SessionStatus {
    pub schema_id: String, pub schema_name: String,
    pub is_composing: bool, pub is_ascii_mode: bool,
    pub is_full_shape: bool, pub is_simplified: bool, pub is_disabled: bool,
}

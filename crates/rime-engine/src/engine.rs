//! Safe Rust API for interacting with a librime engine instance.
//!
//! This is the primary entry point for all IME operations.
//! All unsafe FFI calls are encapsulated within this module.
//! Callers never touch raw pointers or C strings directly.

use crate::candidate::CandidateList;
use crate::error::{RimeError, RimeResult};
use crate::ffi::RimeTraits;
use crate::raw;
use crate::ffi;
use crate::schema::SchemaInfo;
use crate::session::Session;
use skyme_common::{Candidate, Modifiers};
use std::ffi::CString;

/// Whether a keypress was consumed by the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyProcessResult {
    /// Handled — do not pass the key to the application.
    Handled,
    /// Not handled — pass the key through to the application.
    Passthrough,
}

/// High-level, 100% safe API for interacting with a Rime engine instance.
///
/// # Lifecycle
///
/// 1. `Engine::new()` — create a handle (engine not yet active).
/// 2. `Engine::initialize(...)` — initialise librime (must succeed before any other call).
/// 3. Use `create_session()`, `process_key()`, `get_context()`, etc.
/// 4. `Engine::finalize()` — shut down librime (also called on Drop).
///
/// # Thread safety
///
/// librime is internally synchronized. `Engine` is `Send` but not `Sync`.
/// The engine must be finalised on the same thread it was initialised on.
pub struct Engine {
    initialized: bool,
}

impl Engine {
    /// Create a new uninitialised engine handle.
    pub fn new() -> Self {
        Self { initialized: false }
    }

    /// Initialise librime with the given data directories.
    ///
    /// This must succeed before any session-related operation.
    /// Call exactly once — subsequent calls are no-ops.
    ///
    /// # Arguments
    ///
    /// * `shared_data_dir` — Path to Rime's shared data (schemas, dictionaries).  
    ///   Usually `%APPDATA%\Rime` on Windows or `/usr/share/rime-data` on Linux.
    /// * `user_data_dir` — Path to user-specific data (user.yaml, custom schemas).  
    ///   Usually `%APPDATA%\Rime` on Windows or `~/.config/fcitx/rime` on Linux.
    /// * `distribution_name` — Display name for this frontend, e.g. `"Skyme"`.
    pub fn initialize(
        &mut self,
        shared_data_dir: &str,
        user_data_dir: &str,
        distribution_name: &str,
    ) -> RimeResult<()> {
        if self.initialized {
            log::warn!("Rime engine already initialized");
            return Ok(());
        }

        let shared_c = CString::new(shared_data_dir)?;
        let user_c = CString::new(user_data_dir)?;
        let dist_c = CString::new(distribution_name)?;

        let traits = RimeTraits::new(
            shared_c.as_ptr(),
            user_c.as_ptr(),
            dist_c.as_ptr(),
        );

        unsafe {
            raw::setup_logging();
            if !raw::initialize(&traits) {
                return Err(RimeError::InitializeFailed);
            }
        }

        self.initialized = true;
        log::info!(
            "Rime engine initialized: shared={}, user={}",
            shared_data_dir,
            user_data_dir
        );
        Ok(())
    }

    /// Shut down librime and free all resources.
    ///
    /// Called automatically on `Drop`. Safe to call multiple times.
    pub fn finalize(&mut self) {
        if self.initialized {
            unsafe {
                raw::finalize();
            }
            self.initialized = false;
            log::info!("Rime engine finalized");
        }
    }

    /// Check whether the engine has been initialised.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    // ── session management ─────────────────────────────────────────────

    /// Create a new input session.
    ///
    /// Returns `None` if librime failed to allocate a session.
    pub fn create_session(&self) -> RimeResult<Session> {
        self.check_initialized()?;
        unsafe {
            match raw::create_session() {
                Some(id) => Ok(Session::new(id)),
                None => Err(RimeError::SessionFailed(0)),
            }
        }
    }

    /// Check whether a session ID is still valid.
    pub fn find_session(&self, session: &Session) -> bool {
        unsafe { raw::find_session(session.id()) }
    }

    // ── key processing ─────────────────────────────────────────────────

    /// Process a key event within a session.
    ///
    /// Returns `Handled` if the engine consumed the key (composition updated,
    /// candidates changed, etc.) or `Passthrough` if the key should be
    /// forwarded to the application.
    pub fn process_key(
        &self,
        session: &Session,
        keycode: u32,
        modifiers: Modifiers,
    ) -> KeyProcessResult {
        let handled = unsafe { raw::process_key(session.id(), keycode as i32, modifiers.bits() as i32) };
        if handled {
            KeyProcessResult::Handled
        } else {
            KeyProcessResult::Passthrough
        }
    }

    /// Commit the current composition text.
    pub fn commit_composition(&self, session: &Session) {
        unsafe { raw::commit_composition(session.id()) }
    }

    /// Clear the current composition without committing.
    pub fn clear_composition(&self, session: &Session) {
        unsafe { raw::clear_composition(session.id()) }
    }

    // ── context / commit / status ──────────────────────────────────────

    /// Get the current context including preedit and candidates.
    ///
    /// Returns `None` if there is no active composition.
    pub fn get_context(&self, session: &Session) -> RimeResult<Option<SessionContext>> {
        self.check_initialized()?;
        unsafe {
            let mut ctx = match raw::get_context(session.id())? {
                Some(c) => c,
                None => return Ok(None),
            };

            // Copy strings out before freeing.
            let preedit = raw::cstr_to_owned(ctx.composition.preedit);
            let candidates = Self::extract_candidates(&ctx.composition.cand);
            let commit_preview = raw::cstr_to_owned(ctx.commit_text_preview);

            raw::free_context(&mut ctx);

            Ok(Some(SessionContext {
                preedit,
                cursor_pos: ctx.composition.cursor_pos as usize,
                select_labels: ctx.select_label_count,
                commit_text_preview: commit_preview,
                candidates,
            }))
        }
    }

    /// Get the latest committed text.
    pub fn get_commit(&self, session: &Session) -> RimeResult<Option<CommitText>> {
        self.check_initialized()?;
        unsafe {
            let mut commit = match raw::get_commit(session.id())? {
                Some(c) => c,
                None => return Ok(None),
            };

            let text = raw::cstr_to_owned(commit.text).unwrap_or_default();
            raw::free_commit(&mut commit);
            Ok(Some(CommitText { text }))
        }
    }

    /// Get the current engine status for a session.
    pub fn get_status(&self, session: &Session) -> RimeResult<Option<SessionStatus>> {
        self.check_initialized()?;
        unsafe {
            let mut status = match raw::get_status(session.id())? {
                Some(s) => s,
                None => return Ok(None),
            };

            let schema_id = raw::cstr_to_owned(status.schema_id).unwrap_or_default();
            let schema_name = raw::cstr_to_owned(status.schema_name).unwrap_or_default();

            raw::free_status(&mut status);

            Ok(Some(SessionStatus {
                schema_id,
                schema_name,
                is_composing: status.is_composing,
                is_ascii_mode: status.is_ascii_mode,
                is_full_shape: status.is_full_shape,
                is_simplified: status.is_simplified,
                is_disabled: status.is_disabled,
            }))
        }
    }

    // ── candidates ─────────────────────────────────────────────────────

    /// Select a candidate at the given index in the current page.
    pub fn select_candidate(&self, session: &Session, index: u32) -> bool {
        unsafe { raw::select_candidate(session.id(), index as i32) }
    }

    /// Get the full candidate list for a session.
    pub fn get_candidates(&self, session: &Session) -> RimeResult<Option<CandidateList>> {
        self.check_initialized()?;
        unsafe {
            let mut ctx = match raw::get_context(session.id())? {
                Some(c) => c,
                None => return Ok(None),
            };

            let list = Self::extract_candidates(&ctx.composition.cand);
            let page_no = ctx.composition.cand.page_no as u32;
            let page_size = ctx.composition.cand.page_size as u32;
            let is_last = ctx.composition.cand.is_last_page;

            raw::free_context(&mut ctx);

            Ok(Some(CandidateList::new(list, page_no, page_size, is_last)))
        }
    }

    /// Extract candidate entries from a raw RimeCandidateList.
    fn extract_candidates(raw: &ffi::RimeCandidateList) -> Vec<Candidate> {
        if raw.candidates.is_null() || raw.length <= 0 {
            return Vec::new();
        }

        let mut out = Vec::with_capacity(raw.length as usize);
        for i in 0..raw.length as isize {
            unsafe {
                let cand = &*raw.candidates.offset(i);
                out.push(Candidate {
                    text: raw::cstr_to_owned(cand.text).unwrap_or_default(),
                    comment: raw::cstr_to_owned(cand.comment).unwrap_or_default(),
                    index: (raw.candidate_index + i as i32) as u32,
                    quality: 0.0,
                });
            }
        }
        out
    }

    // ── options ────────────────────────────────────────────────────────

    /// Set a boolean option on the session (e.g. `"ascii_mode"`, `"simplification"`).
    pub fn set_option(&self, session: &Session, option: &str, value: bool) -> RimeResult<bool> {
        self.check_initialized()?;
        unsafe { raw::set_option(session.id(), option, value) }
    }

    /// Get a boolean option value from the session.
    pub fn get_option(&self, session: &Session, option: &str) -> RimeResult<bool> {
        self.check_initialized()?;
        unsafe { raw::get_option(session.id(), option) }
    }

    // ── schema ─────────────────────────────────────────────────────────

    /// List all installed schemas.
    pub fn get_schema_list(&self) -> RimeResult<Vec<SchemaInfo>> {
        self.check_initialized()?;
        unsafe {
            let mut list = match raw::get_schema_list()? {
                Some(l) => l,
                None => return Ok(Vec::new()),
            };

            let mut out = Vec::with_capacity(list.length as usize);
            for i in 0..list.length as isize {
                
                    let item = &*list.schemas.offset(i);
                    out.push(SchemaInfo::new(
                        raw::cstr_to_owned(item.schema.schema_id).unwrap_or_default(),
                        raw::cstr_to_owned(item.schema.name).unwrap_or_default(),
                    ));
                
            }
            raw::free_schema_list(&mut list);
            Ok(out)
        }
    }

    /// Switch the active schema for a session.
    pub fn select_schema(&self, session: &Session, schema_id: &str) -> RimeResult<bool> {
        self.check_initialized()?;
        unsafe { raw::select_schema(session.id(), schema_id) }
    }

    /// Get the currently active schema for a session.
    pub fn current_schema(&self, session: &Session) -> RimeResult<Option<String>> {
        self.check_initialized()?;
        unsafe {
            let schema = match raw::current_schema(session.id())? {
                Some(s) => s,
                None => return Ok(None),
            };
            let name = raw::cstr_to_owned(schema.name).unwrap_or_default();
            Ok(Some(name))
        }
    }

    // ── internal helpers ───────────────────────────────────────────────

    fn check_initialized(&self) -> RimeResult<()> {
        if !self.initialized {
            Err(RimeError::NotInitialized)
        } else {
            Ok(())
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.finalize();
    }
}

// ── Data types returned by the safe API ─────────────────────────────────────

/// Snapshot of a session's context (preedit + candidates + preview).
#[derive(Debug, Clone)]
pub struct SessionContext {
    /// Preedit / composition text, if any.
    pub preedit: Option<String>,
    /// Cursor position within the preedit.
    pub cursor_pos: usize,
    /// Number of select-label keys for the current page.
    pub select_labels: i32,
    /// Text that will be committed when the composition ends.
    pub commit_text_preview: Option<String>,
    /// Current page of candidates.
    pub candidates: Vec<Candidate>,
}

/// Text committed by the engine.
#[derive(Debug, Clone)]
pub struct CommitText {
    pub text: String,
}

/// Snapshot of a session's status flags.
#[derive(Debug, Clone)]
pub struct SessionStatus {
    pub schema_id: String,
    pub schema_name: String,
    pub is_composing: bool,
    pub is_ascii_mode: bool,
    pub is_full_shape: bool,
    pub is_simplified: bool,
    pub is_disabled: bool,
}

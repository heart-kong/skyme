//! Raw FFI declarations matching librime's `rime.h`.
//!
//! These mirror the C API exactly. Use `raw.rs` for safe wrappers.
//! Struct definitions match librime commit e3a8f6b (1.x API).

pub use std::ffi::c_char;

// ── RimeTraits  (initialization configuration) ──────────────────────────────

#[repr(C)]
#[derive(Debug, Clone)]
pub struct RimeTraits {
    pub data_size: i32,
    pub shared_data_dir: *const c_char,
    pub user_data_dir: *const c_char,
    pub distribution_name: *const c_char,
    pub distribution_code_name: *const c_char,
    pub distribution_version: *const c_char,
    pub app_name: *const c_char,
    pub reserved: [*const c_char; 8usize],
}

impl RimeTraits {
    pub fn new(
        shared_data_dir: *const c_char,
        user_data_dir: *const c_char,
        distribution_name: *const c_char,
    ) -> Self {
        Self {
            data_size: size_of::<RimeTraits>() as i32,
            shared_data_dir,
            user_data_dir,
            distribution_name,
            distribution_code_name: std::ptr::null(),
            distribution_version: std::ptr::null(),
            app_name: std::ptr::null(),
            reserved: [std::ptr::null(); 8],
        }
    }
}

// ── RimeContext / Composition / Candidates ──────────────────────────────────

#[repr(C)]
#[derive(Debug, Clone)]
pub struct RimeContext {
    pub data_size: i32,
    pub composition: RimeComposition,
    pub commit_text_preview: *mut c_char,
    pub select_labels: *mut *mut c_char,
    pub select_label_count: i32,
}

impl Default for RimeContext {
    fn default() -> Self {
        Self {
            data_size: size_of::<RimeContext>() as i32,
            composition: RimeComposition::default(),
            commit_text_preview: std::ptr::null_mut(),
            select_labels: std::ptr::null_mut(),
            select_label_count: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct RimeComposition {
    pub length: i32,
    pub cursor_pos: i32,
    pub sel_start: i32,
    pub sel_end: i32,
    pub preedit: *mut c_char,
    pub cand: RimeCandidateList,
}

impl Default for RimeComposition {
    fn default() -> Self {
        Self {
            length: 0,
            cursor_pos: 0,
            sel_start: 0,
            sel_end: 0,
            preedit: std::ptr::null_mut(),
            cand: RimeCandidateList::default(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct RimeCandidateList {
    pub data_size: i32,
    pub length: i32,
    pub candidate_index: i32,
    pub candidates: *mut RimeCandidate,
    pub page_size: i32,
    pub page_no: i32,
    pub is_last_page: bool,
    pub is_complete: bool,
    pub select_keys: *mut c_char,
    pub current_page_start: i32,
}

impl Default for RimeCandidateList {
    fn default() -> Self {
        Self {
            data_size: size_of::<RimeCandidateList>() as i32,
            length: 0,
            candidate_index: 0,
            candidates: std::ptr::null_mut(),
            page_size: 5,
            page_no: 0,
            is_last_page: true,
            is_complete: true,
            select_keys: std::ptr::null_mut(),
            current_page_start: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct RimeCandidate {
    pub data_size: i32,
    pub text: *mut c_char,
    pub comment: *mut c_char,
}

impl Default for RimeCandidate {
    fn default() -> Self {
        Self {
            data_size: size_of::<RimeCandidate>() as i32,
            text: std::ptr::null_mut(),
            comment: std::ptr::null_mut(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct RimeCommit {
    pub data_size: i32,
    pub text: *mut c_char,
}

impl Default for RimeCommit {
    fn default() -> Self {
        Self {
            data_size: size_of::<RimeCommit>() as i32,
            text: std::ptr::null_mut(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct RimeStatus {
    pub data_size: i32,
    pub schema_id: *mut c_char,
    pub schema_name: *mut c_char,
    pub is_disabled: bool,
    pub is_composing: bool,
    pub is_ascii_mode: bool,
    pub is_full_shape: bool,
    pub is_simplified: bool,
    pub is_traditional: bool,
    pub is_ascii_punct: bool,
}

impl Default for RimeStatus {
    fn default() -> Self {
        Self {
            data_size: size_of::<RimeStatus>() as i32,
            schema_id: std::ptr::null_mut(),
            schema_name: std::ptr::null_mut(),
            is_disabled: false,
            is_composing: false,
            is_ascii_mode: false,
            is_full_shape: false,
            is_simplified: false,
            is_traditional: false,
            is_ascii_punct: false,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct RimeSchema {
    pub data_size: i32,
    pub schema_id: *mut c_char,
    pub name: *mut c_char,
    pub reserved: *mut c_char,
}

impl Default for RimeSchema {
    fn default() -> Self {
        Self {
            data_size: size_of::<RimeSchema>() as i32,
            schema_id: std::ptr::null_mut(),
            name: std::ptr::null_mut(),
            reserved: std::ptr::null_mut(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct RimeSchemaList {
    pub data_size: i32,
    pub length: i32,
    pub schemas: *mut RimeSchemaListItem,
}

impl Default for RimeSchemaList {
    fn default() -> Self {
        Self {
            data_size: size_of::<RimeSchemaList>() as i32,
            length: 0,
            schemas: std::ptr::null_mut(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct RimeSchemaListItem {
    pub schema: RimeSchema,
    pub next: *mut RimeSchemaListItem,
}

impl Default for RimeSchemaListItem {
    fn default() -> Self {
        Self {
            schema: RimeSchema::default(),
            next: std::ptr::null_mut(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct RimePreedit {
    pub data_size: i32,
    pub length: i32,
    pub cursor_pos: i32,
    pub sel_start: i32,
    pub sel_end: i32,
    pub preedit: *mut c_char,
}

impl Default for RimePreedit {
    fn default() -> Self {
        Self {
            data_size: size_of::<RimePreedit>() as i32,
            length: 0,
            cursor_pos: 0,
            sel_start: 0,
            sel_end: 0,
            preedit: std::ptr::null_mut(),
        }
    }
}

// ── extern "C" functions ────────────────────────────────────────────────────
//
// These are resolved at link time against librime (rime.dll / librime.so / librime.dylib).
// Without librime linked, `cargo build` will fail, but `cargo check` succeeds
// since it does not run the linker.

extern "C" {
    // ── lifecycle ──
    pub fn RimeSetupLogging();
    pub fn RimeInitialize(traits: *const RimeTraits) -> bool;
    pub fn RimeFinalize();
    pub fn RimeStartMaintenance(full_check: bool) -> bool;

    // ── sessions ──
    pub fn RimeCreateSession() -> u64;
    pub fn RimeFindSession(session_id: u64) -> bool;
    pub fn RimeDestroySession(session_id: u64);
    pub fn RimeCleanupStaleSessions();
    pub fn RimeCleanupAllSessions();

    // ── key processing ──
    pub fn RimeProcessKey(session_id: u64, keycode: i32, modifiers: i32) -> bool;
    pub fn RimeCommitComposition(session_id: u64);
    pub fn RimeClearComposition(session_id: u64);

    // ── context / commit / status ──
    pub fn RimeGetContext(session_id: u64, ctx: *mut RimeContext) -> bool;
    pub fn RimeFreeContext(ctx: *mut RimeContext);
    pub fn RimeGetCommit(session_id: u64, commit: *mut RimeCommit) -> bool;
    pub fn RimeFreeCommit(commit: *mut RimeCommit);
    pub fn RimeGetStatus(session_id: u64, status: *mut RimeStatus) -> bool;
    pub fn RimeFreeStatus(status: *mut RimeStatus);

    // ── candidates ──
    pub fn RimeSelectCandidate(session_id: u64, index: i32) -> bool;
    pub fn RimeCandidateListFromIndex(session_id: u64, index: i32, list: *mut RimeCandidateList) -> bool;
    pub fn RimeFreeCandidateList(list: *mut RimeCandidateList);

    // ── options ──
    pub fn RimeSetOption(session_id: u64, option: *const c_char, value: bool) -> bool;
    pub fn RimeGetOption(session_id: u64, option: *const c_char) -> bool;

    // ── schema ──
    pub fn RimeGetSchemaList(list: *mut RimeSchemaList) -> bool;
    pub fn RimeFreeSchemaList(list: *mut RimeSchemaList);
    pub fn RimeGetSchemaById(schema_id: *const c_char) -> bool;
    pub fn RimeSelectSchema(session_id: u64, schema_id: *const c_char) -> bool;
    pub fn RimeCurrentSchema(session_id: u64, schema: *mut RimeSchema) -> bool;

    // ── preedit ──
    pub fn RimeGetPreedit(session_id: u64, preedit: *mut RimePreedit) -> bool;
    pub fn RimeFreePreedit(preedit: *mut RimePreedit);

    // ── deployer ──
    pub fn RimeDeployer_Initialize(traits: *const RimeTraits) -> bool;
    pub fn RimeDeployer_Shutdown();
    pub fn RimeDeployer_StartMaintenance(full_check: bool);
    pub fn RimeDeployer_IsMaintenanceRunning() -> bool;
    pub fn RimeDeployer_JoinMaintenanceThread();
}

//! FFI declarations for librime.
//!
//! Contains C struct definitions and a [`RimeApi`] struct that holds
//! function pointers loaded at runtime via `libloading`.

use libloading::Library;
use std::ffi::c_char;
use std::sync::Arc;

include!("ffi_structs.rs.in");

/// Runtime-loaded librime API.
pub struct RimeApi {
    pub(crate) _lib: Arc<Library>,
    pub RimeSetupLogging: unsafe extern "C" fn(),
    pub RimeInitialize: unsafe extern "C" fn(*const RimeTraits) -> bool,
    pub RimeFinalize: unsafe extern "C" fn(),
    pub RimeStartMaintenance: unsafe extern "C" fn(bool) -> bool,
    pub RimeCreateSession: unsafe extern "C" fn() -> u64,
    pub RimeFindSession: unsafe extern "C" fn(u64) -> bool,
    pub RimeDestroySession: unsafe extern "C" fn(u64),
    pub RimeCleanupStaleSessions: unsafe extern "C" fn(),
    pub RimeCleanupAllSessions: unsafe extern "C" fn(),
    pub RimeProcessKey: unsafe extern "C" fn(u64, i32, i32) -> bool,
    pub RimeCommitComposition: unsafe extern "C" fn(u64),
    pub RimeClearComposition: unsafe extern "C" fn(u64),
    pub RimeGetContext: unsafe extern "C" fn(u64, *mut RimeContext) -> bool,
    pub RimeFreeContext: unsafe extern "C" fn(*mut RimeContext),
    pub RimeGetCommit: unsafe extern "C" fn(u64, *mut RimeCommit) -> bool,
    pub RimeFreeCommit: unsafe extern "C" fn(*mut RimeCommit),
    pub RimeGetStatus: unsafe extern "C" fn(u64, *mut RimeStatus) -> bool,
    pub RimeFreeStatus: unsafe extern "C" fn(*mut RimeStatus),
    pub RimeSelectCandidate: unsafe extern "C" fn(u64, i32) -> bool,
    pub RimeCandidateListFromIndex: unsafe extern "C" fn(u64, i32, *mut RimeCandidateList) -> bool,
    pub RimeFreeCandidateList: unsafe extern "C" fn(*mut RimeCandidateList),
    pub RimeSetOption: unsafe extern "C" fn(u64, *const c_char, bool) -> bool,
    pub RimeGetOption: unsafe extern "C" fn(u64, *const c_char) -> bool,
    pub RimeGetSchemaList: unsafe extern "C" fn(*mut RimeSchemaList) -> bool,
    pub RimeFreeSchemaList: unsafe extern "C" fn(*mut RimeSchemaList),
    pub RimeGetSchemaById: unsafe extern "C" fn(*const c_char) -> bool,
    pub RimeSelectSchema: unsafe extern "C" fn(u64, *const c_char) -> bool,
    pub RimeCurrentSchema: unsafe extern "C" fn(u64, *mut RimeSchema) -> bool,
    pub RimeGetPreedit: unsafe extern "C" fn(u64, *mut RimePreedit) -> bool,
    pub RimeFreePreedit: unsafe extern "C" fn(*mut RimePreedit),
    pub RimeDeployer_Initialize: unsafe extern "C" fn(*const RimeTraits) -> bool,
    pub RimeDeployer_Shutdown: unsafe extern "C" fn(),
    pub RimeDeployer_StartMaintenance: unsafe extern "C" fn(bool),
    pub RimeDeployer_IsMaintenanceRunning: unsafe extern "C" fn() -> bool,
    pub RimeDeployer_JoinMaintenanceThread: unsafe extern "C" fn(),
}

impl RimeApi {
    pub unsafe fn new(lib: Arc<Library>) -> Result<Self, libloading::Error> {
        macro_rules! load { ($lib:expr, $name:ident) => { $lib.get(stringify!($name).as_bytes()).map(|s| *s)? }; }
        Ok(Self {
            _lib: lib.clone(),
            RimeSetupLogging: load!(lib, RimeSetupLogging),
            RimeInitialize: load!(lib, RimeInitialize),
            RimeFinalize: load!(lib, RimeFinalize),
            RimeStartMaintenance: load!(lib, RimeStartMaintenance),
            RimeCreateSession: load!(lib, RimeCreateSession),
            RimeFindSession: load!(lib, RimeFindSession),
            RimeDestroySession: load!(lib, RimeDestroySession),
            RimeCleanupStaleSessions: load!(lib, RimeCleanupStaleSessions),
            RimeCleanupAllSessions: load!(lib, RimeCleanupAllSessions),
            RimeProcessKey: load!(lib, RimeProcessKey),
            RimeCommitComposition: load!(lib, RimeCommitComposition),
            RimeClearComposition: load!(lib, RimeClearComposition),
            RimeGetContext: load!(lib, RimeGetContext),
            RimeFreeContext: load!(lib, RimeFreeContext),
            RimeGetCommit: load!(lib, RimeGetCommit),
            RimeFreeCommit: load!(lib, RimeFreeCommit),
            RimeGetStatus: load!(lib, RimeGetStatus),
            RimeFreeStatus: load!(lib, RimeFreeStatus),
            RimeSelectCandidate: load!(lib, RimeSelectCandidate),
            RimeCandidateListFromIndex: load!(lib, RimeCandidateListFromIndex),
            RimeFreeCandidateList: load!(lib, RimeFreeCandidateList),
            RimeSetOption: load!(lib, RimeSetOption),
            RimeGetOption: load!(lib, RimeGetOption),
            RimeGetSchemaList: load!(lib, RimeGetSchemaList),
            RimeFreeSchemaList: load!(lib, RimeFreeSchemaList),
            RimeGetSchemaById: load!(lib, RimeGetSchemaById),
            RimeSelectSchema: load!(lib, RimeSelectSchema),
            RimeCurrentSchema: load!(lib, RimeCurrentSchema),
            RimeGetPreedit: load!(lib, RimeGetPreedit),
            RimeFreePreedit: load!(lib, RimeFreePreedit),
            RimeDeployer_Initialize: load!(lib, RimeDeployer_Initialize),
            RimeDeployer_Shutdown: load!(lib, RimeDeployer_Shutdown),
            RimeDeployer_StartMaintenance: load!(lib, RimeDeployer_StartMaintenance),
            RimeDeployer_IsMaintenanceRunning: load!(lib, RimeDeployer_IsMaintenanceRunning),
            RimeDeployer_JoinMaintenanceThread: load!(lib, RimeDeployer_JoinMaintenanceThread),
        })
    }
}

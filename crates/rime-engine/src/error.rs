//! Error types for the Rime engine.

use std::ffi::NulError;

/// Errors that can occur during Rime engine operations.
#[derive(Debug)]
pub enum RimeError {
    /// librime was not initialized (RimeInitialize not called).
    NotInitialized,
    /// Initialization failed (RimeInitialize returned false).
    InitializeFailed,
    /// A session operation failed (invalid / stale session).
    SessionFailed(u64),
    /// Failed to convert a Rust string to a C string (embedded NUL).
    NulError(NulError),
    /// Failed to allocate or query a Rime struct.
    ApiCallFailed(&'static str),
    /// Deployment failed.
    DeployFailed(String),
    /// A retrieved string contained invalid UTF-8.
    InvalidUtf8(std::str::Utf8Error),
}

impl std::fmt::Display for RimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RimeError::NotInitialized => write!(f, "Rime engine not initialized"),
            RimeError::InitializeFailed => write!(f, "RimeInitialize returned false"),
            RimeError::SessionFailed(id) => write!(f, "Session {} operation failed", id),
            RimeError::NulError(e) => write!(f, "NUL in string: {}", e),
            RimeError::ApiCallFailed(api) => write!(f, "Rime API call failed: {}", api),
            RimeError::DeployFailed(msg) => write!(f, "Deployment failed: {}", msg),
            RimeError::InvalidUtf8(e) => write!(f, "Invalid UTF-8: {}", e),
        }
    }
}

impl std::error::Error for RimeError {}

impl From<NulError> for RimeError {
    fn from(e: NulError) -> Self {
        RimeError::NulError(e)
    }
}

impl From<std::str::Utf8Error> for RimeError {
    fn from(e: std::str::Utf8Error) -> Self {
        RimeError::InvalidUtf8(e)
    }
}

/// Convenience alias.
pub type RimeResult<T> = Result<T, RimeError>;

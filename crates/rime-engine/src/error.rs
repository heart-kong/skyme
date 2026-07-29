//! Error types for the Rime engine.

use std::ffi::NulError;

/// Errors that can occur during Rime engine operations.
#[derive(Debug)]
pub enum RimeError {
    /// librime was not initialized.
    NotInitialized,
    /// RimeInitialize returned false.
    InitializeFailed,
    /// A session operation failed.
    SessionFailed(u64),
    /// Failed to convert a Rust string to a C string (embedded NUL).
    NulError(NulError),
    /// A Rime API call returned failure.
    ApiCallFailed(&'static str),
    /// Failed to load librime dynamically via libloading.
    LibraryLoadFailed(String),
    /// Deployment failed.
    DeployFailed(String),
    /// Retrieved string contained invalid UTF-8.
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
            RimeError::LibraryLoadFailed(msg) => write!(f, "Failed to load librime: {}", msg),
            RimeError::DeployFailed(msg) => write!(f, "Deployment failed: {}", msg),
            RimeError::InvalidUtf8(e) => write!(f, "Invalid UTF-8: {}", e),
        }
    }
}

impl std::error::Error for RimeError {}

impl From<NulError> for RimeError { fn from(e: NulError) -> Self { RimeError::NulError(e) } }
impl From<std::str::Utf8Error> for RimeError { fn from(e: std::str::Utf8Error) -> Self { RimeError::InvalidUtf8(e) } }

pub type RimeResult<T> = Result<T, RimeError>;

#[cfg(test)]
mod tests {
    use crate::error::*;

    #[test]
    fn test_error_display() {
        let e = RimeError::NotInitialized;
        assert_eq!(format!("{}", e), "Rime engine not initialized");
    }

    #[test]
    fn test_initialize_failed() {
        let e = RimeError::InitializeFailed;
        assert_eq!(format!("{}", e), "RimeInitialize returned false");
    }

    #[test]
    fn test_session_failed() {
        let e = RimeError::SessionFailed(42);
        assert_eq!(format!("{}", e), "Session 42 operation failed");
    }

    #[test]
    fn test_api_call_failed() {
        let e = RimeError::ApiCallFailed("RimeProcessKey");
        assert_eq!(format!("{}", e), "Rime API call failed: RimeProcessKey");
    }

    #[test]
    fn test_nul_error() {
        use std::ffi::CString;
        let result = CString::new("hello\0world");
        assert!(result.is_err());
        let e = RimeError::from(result.unwrap_err());
        assert!(format!("{}", e).contains("NUL"));
    }

    #[test]
    fn test_library_load_failed() {
        let e = RimeError::LibraryLoadFailed("cannot find library".into());
        assert!(format!("{}", e).contains("cannot find library"));
    }

    #[test]
    fn test_deploy_failed() {
        let e = RimeError::DeployFailed("timeout".into());
        assert_eq!(format!("{}", e), "Deployment failed: timeout");
    }

    #[test]
    fn test_utf8_error() {
        let invalid = &[0xFF, 0xFE, 0x00];
        let result = std::str::from_utf8(invalid);
        assert!(result.is_err());
        let e = RimeError::from(result.unwrap_err());
        assert!(format!("{}", e).contains("UTF-8"));
    }

    #[test]
    fn test_error_is_std_error() {
        use std::error::Error;
        let e = RimeError::NotInitialized;
        let _: &dyn Error = &e;
    }

    #[test]
    fn test_result_alias() {
        let r: RimeResult<i32> = Ok(42);
        assert_eq!(r.unwrap(), 42);
    }
}

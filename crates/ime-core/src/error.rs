//! Error types for IME operations.

use thiserror::Error;

/// Errors that can occur during IME / TSF operations.
#[derive(Error, Debug)]
pub enum ImeError {
    #[error("TSF activation failed: {0}")]
    ActivationFailed(String),
    #[error("TSF deactivation failed: {0}")]
    DeactivationFailed(String),
    #[error("COM error: HRESULT = 0x{0:x}")]
    ComError(i32),
    #[error("Engine error: {0}")]
    Engine(String),
}

pub type ImeResult<T> = Result<T, ImeError>;

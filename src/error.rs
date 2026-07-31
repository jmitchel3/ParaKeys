//! Shared error types for ParaKeys.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParaKeysError {
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),

    #[error("vault not found at {0}")]
    VaultNotFound(String),

    #[error("invalid vault: {0}")]
    InvalidVault(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

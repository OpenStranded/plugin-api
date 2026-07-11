use thiserror::Error;

/// Errors that occur during Service API calls or Registry access.
///
/// All variants are Clone + Debug for passing across the WASM boundary.
#[derive(Clone, Debug, Error)]
pub enum ServiceError {
    /// An argument did not match the expected type.
    #[error("invalid arguments: expected {expected}, got {found}")]
    TypeMismatch {
        expected: String,
        found: String,
    },

    /// Method not found on the service.
    #[error("unknown method: {0}")]
    UnknownMethod(String),

    /// Service domain is not registered.
    #[error("service domain not found: {0}")]
    DomainNotFound(String),

    /// File not found in Registry at domain/filename.
    #[error("file not found in registry: {domain}/{filename}")]
    FileNotFound {
        domain: String,
        filename: String,
    },

    /// Registry domain not found.
    #[error("registry domain not found: {0}")]
    RegistryDomainNotFound(String),

    /// Error parsing registry data (RON / JSON / binary).
    #[error("parse error: {0}")]
    ParseError(String),

    /// Internal / generic error.
    #[error("internal error: {0}")]
    Internal(String),
}

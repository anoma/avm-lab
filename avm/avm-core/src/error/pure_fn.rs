//! Pure function layer errors.

#[derive(Debug, thiserror::Error)]
pub enum PureError {
    #[error("function not found: {0}")]
    NotFound(String),

    #[error("function already registered: {0}")]
    AlreadyRegistered(String),

    #[error("version conflict for function: {0}")]
    VersionConflict(String),
}

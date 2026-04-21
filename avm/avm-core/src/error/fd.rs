//! Finite-domain and nondeterminism errors.

#[derive(Debug, thiserror::Error)]
pub enum FdError {
    #[error("FD constraint solver not implemented: {0}")]
    NotImplemented(String),
}

#[derive(Debug, thiserror::Error)]
pub enum NondetError {
    #[error("nondeterminism not implemented: {0}")]
    NotImplemented(String),
}

//! Reification-layer errors.

#[derive(Debug, thiserror::Error)]
pub enum ReifyError {
    #[error("failed to reify execution context")]
    ContextFailed,

    #[error("no active transaction to reify")]
    NoTransaction,

    #[error("transaction state access denied")]
    TxStateAccessDenied,

    #[error("constraint store unavailable")]
    ConstraintStoreUnavailable,
}

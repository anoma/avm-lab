//! Transaction-layer errors.

use crate::types::TxId;

#[derive(Debug, thiserror::Error)]
pub enum TxError {
    #[error("transaction conflict: {0}")]
    Conflict(TxId),

    #[error("transaction not found: {0}")]
    NotFound(TxId),

    #[error("transaction already committed: {0}")]
    AlreadyCommitted(TxId),

    #[error("transaction already aborted: {0}")]
    AlreadyAborted(TxId),

    #[error("no active transaction")]
    NoActiveTx,

    #[error("operation invalid during active transaction")]
    InvalidDuringTx,
}

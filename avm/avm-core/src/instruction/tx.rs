//! Transaction instructions: begin, commit, abort.

use crate::types::{ControllerId, TxId};

/// Instructions for transaction management.
///
/// Transactions provide serializable snapshot isolation. All writes within
/// a transaction are buffered and applied atomically on commit.
#[derive(Debug)]
pub enum TxInstruction {
    /// Start a new transaction with an optional controller.
    Begin(Option<ControllerId>),
    /// Commit the transaction: validate the read set, then apply all
    /// pending writes atomically.
    Commit(TxId),
    /// Abort the transaction: discard all pending writes.
    Abort(TxId),
}

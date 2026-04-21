//! AVM Examples — demonstration programs for the Anoma Virtual Machine.
//!
//! Contains `PingPong` and Battleship example programs that exercise the AVM
//! instruction set, along with integration tests that verify end-to-end
//! execution against the formal specification.

pub mod battleship;
pub mod ping_pong;
pub mod protocol;

use avm_core::types::{ObjectId, TxId, Val};

/// Extract an `ObjectId` from a `Val::ObjectRef`, returning `None` on wrong type.
pub fn object_id_from_val(val: &Val) -> Option<ObjectId> {
    val.as_object_id()
}

/// Extract a `TxId` from a `Val::TxRef`, returning `None` on wrong type.
pub fn tx_id_from_val(val: &Val) -> Option<TxId> {
    val.as_tx_id()
}

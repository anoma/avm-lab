//! Pluggable transport for remote object calls.

use crate::error::AVMError;
use crate::types::{MachineId, ObjectId, Val};

/// Transport layer for routing calls to remote machines.
///
/// The interpreter invokes this when `target_machine != local_machine`.
pub trait Transport: Send + Sync {
    fn remote_call(
        &self,
        target_machine: &MachineId,
        target: ObjectId,
        input: Val,
        sender: ObjectId,
    ) -> Result<Val, AVMError>;
}

/// Null transport: every remote call fails with Unreachable.
pub struct LocalOnlyTransport;

impl Transport for LocalOnlyTransport {
    fn remote_call(
        &self,
        target_machine: &MachineId,
        _target: ObjectId,
        _input: Val,
        _sender: ObjectId,
    ) -> Result<Val, AVMError> {
        Err(crate::error::MachineError::Unreachable(target_machine.clone()).into())
    }
}

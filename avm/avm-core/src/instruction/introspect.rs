//! Introspection instructions: self, input, state, sender, machine.

use crate::types::Val;

/// Instructions for querying the current execution context.
///
/// All introspection instructions are safe — they read state but do not
/// modify the global store.
#[derive(Debug)]
pub enum IntrospectInstruction {
    /// Get the current object's ID.
    GetSelf,
    /// Get the current input message.
    GetInput,
    /// Get the current physical machine.
    GetCurrentMachine,
    /// Read the object's internal state.
    GetState,
    /// Replace the object's internal state.
    SetState(Vec<Val>),
    /// Get the sender of the current call.
    GetSender,
}

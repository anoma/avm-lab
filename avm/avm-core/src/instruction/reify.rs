//! Reification instructions: capture execution context as data.

/// Instructions for reifying the execution state into inspectable values.
#[derive(Debug)]
pub enum ReifyInstruction {
    /// Capture the current execution context (safe).
    ReifyContext,
    /// Capture the active transaction state (unsafe).
    ReifyTxState,
    /// Capture the constraint solver state (unsafe).
    ReifyConstraints,
}

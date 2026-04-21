//! Pure function instructions: call, register, update.

use crate::types::Val;

/// Instructions for the extensible pure function registry.
#[derive(Debug)]
pub enum PureInstruction {
    /// Call a named pure function with arguments.
    Call { name: String, args: Vec<Val> },
    /// Register a new pure function (unsafe, fails if name exists).
    Register { name: String, function_id: u64 },
    /// Update an existing pure function (increments version).
    Update { name: String, function_id: u64 },
}

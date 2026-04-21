//! Nondeterminism instructions.

use crate::types::constraint::NondetConstraint;
use crate::types::Val;

/// Instructions for nondeterministic choice.
///
/// Not yet implemented in the interpreter — all operations return
/// `Err(NondetError::NotImplemented)`.
#[derive(Debug)]
pub enum NondetInstruction {
    /// Choose from a set of values (deferred until commit).
    Choose(Vec<Val>),
    /// Assert a constraint (accumulated and validated at commit).
    Require(NondetConstraint),
}

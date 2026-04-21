//! Finite-domain constraint instructions.

use crate::types::constraint::{Constraint, Domain, VarId};

/// Instructions for the finite-domain constraint solver.
///
/// Not yet implemented in the interpreter — all operations return
/// `Err(FdError::NotImplemented)`.
#[derive(Debug)]
pub enum FdInstruction {
    /// Create a new variable with a finite domain.
    NewVar(Domain),
    /// Narrow a variable's domain by intersection.
    Narrow { var: VarId, domain: Domain },
    /// Post a relational constraint and trigger propagation.
    Post(Constraint),
    /// Select a value for a variable (call-time choice).
    Label(VarId),
}

//! Finite-domain constraint types for the AVM's constraint programming layer.
//!
//! These types mirror the Agda specification's `FDInstruction` and
//! `NondetInstruction` supporting types.

use super::Val;

/// Identifier for a constraint variable.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VarId(pub u64);

/// A finite domain: the set of possible values a variable can take.
pub type Domain = Vec<Val>;

/// Relational constraint between variables or between a variable and a value.
#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    Eq(VarId, VarId),
    Neq(VarId, VarId),
    Leq(VarId, VarId),
    Lt(VarId, VarId),
    Geq(VarId, VarId),
    Gt(VarId, VarId),
    AllDiff(Vec<VarId>),
    ValEq(VarId, Val),
    ValLeq(VarId, Val),
    ValLt(VarId, Val),
    ValGeq(VarId, Val),
    ValGt(VarId, Val),
}

/// Constraint used by the nondeterminism layer.
#[derive(Debug, Clone, PartialEq)]
pub enum NondetConstraint {
    Assert(bool),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constraint_variants() {
        let c = Constraint::AllDiff(vec![VarId(0), VarId(1), VarId(2)]);
        assert!(matches!(c, Constraint::AllDiff(vars) if vars.len() == 3));
    }

    #[test]
    fn var_id_equality() {
        assert_eq!(VarId(0), VarId(0));
        assert_ne!(VarId(0), VarId(1));
    }
}

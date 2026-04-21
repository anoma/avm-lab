//! Core value types, identifiers, and metadata for the AVM.

pub mod constraint;
pub mod ids;
pub mod meta;
pub mod val;

pub use constraint::{Constraint, Domain, NondetConstraint, VarId};
pub use ids::{ControllerId, FreshIdGen, MachineId, ObjectId, TxId};
pub use meta::{ObjectMeta, ReifiedConstraints, ReifiedContext, ReifiedTxState};
pub use val::{Input, Output, Val};

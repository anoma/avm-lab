//! Object metadata and reification types.
//!
//! These types capture the runtime metadata associated with AVM objects and
//! the reified execution context available through introspection instructions.

use super::{ControllerId, Input, MachineId, ObjectId, TxId, Val};

/// Runtime metadata for an AVM object.
///
/// Every object carries immutable provenance (creating controller) and mutable
/// ownership (current controller). The machine field tracks physical location.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ObjectMeta {
    pub object_id: ObjectId,
    pub machine: MachineId,
    pub creating_controller: Option<ControllerId>,
    pub current_controller: Option<ControllerId>,
}

/// Snapshot of the current execution context, produced by `reifyContext`.
///
/// This is a pure data representation of the interpreter's execution frame,
/// made available to programs for meta-programming.
#[derive(Debug, Clone, PartialEq)]
pub struct ReifiedContext {
    pub self_id: ObjectId,
    pub input: Input,
    pub sender: Option<ObjectId>,
    pub machine: MachineId,
    pub controller: Option<ControllerId>,
}

/// Snapshot of the active transaction's pending writes, produced by `reifyTxState`.
#[derive(Debug, Clone, PartialEq)]
pub struct ReifiedTxState {
    pub tx_id: TxId,
    pub writes: Vec<(ObjectId, Input)>,
    pub creates: Vec<ObjectId>,
    pub destroys: Vec<ObjectId>,
    pub observed: Vec<ObjectId>,
}

/// Snapshot of the constraint solver state, produced by `reifyConstraints`.
#[derive(Debug, Clone, PartialEq)]
pub struct ReifiedConstraints {
    pub variable_count: u64,
    pub domains: Vec<(super::VarId, Vec<Val>)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_meta_construction() {
        let meta = ObjectMeta {
            object_id: ObjectId(1),
            machine: MachineId("m0".into()),
            creating_controller: Some(ControllerId("c0".into())),
            current_controller: Some(ControllerId("c0".into())),
        };
        assert_eq!(meta.object_id, ObjectId(1));
        assert_eq!(meta.creating_controller, meta.current_controller);
    }

    #[test]
    fn reified_context_construction() {
        let ctx = ReifiedContext {
            self_id: ObjectId(0),
            input: Val::Nat(42),
            sender: None,
            machine: MachineId("local".into()),
            controller: None,
        };
        assert!(ctx.sender.is_none());
        assert_eq!(ctx.input, Val::Nat(42));
    }
}

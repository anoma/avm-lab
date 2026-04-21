//! The AVM instruction set architecture.
//!
//! Instructions are organized into composable layers that mirror the formal
//! specification. Each layer adds capabilities on top of the previous one:
//!
//! - **Layer 0**: Objects, introspection, reflection, reification
//! - **Layer 1**: Transactions
//! - **Layer 2**: Pure functions
//! - **Layer 3**: Machine and controller distribution
//! - **Layer 4**: Finite-domain constraints and nondeterminism

pub mod controller;
pub mod fd;
pub mod introspect;
pub mod machine;
pub mod nondet;
pub mod obj;
pub mod pure_fn;
pub mod reflect;
pub mod reify;
pub mod tx;

pub use controller::ControllerInstruction;
pub use fd::FdInstruction;
pub use introspect::IntrospectInstruction;
pub use machine::MachineInstruction;
pub use nondet::NondetInstruction;
pub use obj::ObjInstruction;
pub use pure_fn::PureInstruction;
pub use reflect::ReflectInstruction;
pub use reify::ReifyInstruction;
pub use tx::TxInstruction;

use crate::types::{ControllerId, Input, ObjectId, Val};

/// The complete AVM instruction set, unifying all layers.
///
/// The interpreter dispatches on this enum to execute each instruction.
/// The variants correspond to the layered families in the specification.
#[derive(Debug)]
pub enum Instruction {
    Obj(ObjInstruction),
    Introspect(IntrospectInstruction),
    Reflect(ReflectInstruction),
    Reify(ReifyInstruction),
    Tx(TxInstruction),
    Pure(PureInstruction),
    Machine(MachineInstruction),
    Controller(ControllerInstruction),
    Fd(FdInstruction),
    Nondet(NondetInstruction),
}

// --- Convenience constructors matching the spec's flat namespace ---

/// Create an object by behavior name, with an optional controller.
pub fn create_obj(name: impl Into<String>, controller: Option<ControllerId>) -> Instruction {
    Instruction::Obj(ObjInstruction::Create {
        behavior_name: name.into(),
        controller,
    })
}

/// Mark an object for destruction.
pub fn destroy_obj(id: ObjectId) -> Instruction {
    Instruction::Obj(ObjInstruction::Destroy(id))
}

/// Synchronous message-passing call to an object.
pub fn call(target: ObjectId, input: Input) -> Instruction {
    Instruction::Obj(ObjInstruction::Call { target, input })
}

/// Receive the next message (platform-specific blocking).
pub fn receive() -> Instruction {
    Instruction::Obj(ObjInstruction::Receive)
}

/// Query the current object's ID.
pub fn get_self() -> Instruction {
    Instruction::Introspect(IntrospectInstruction::GetSelf)
}

/// Query the current input message.
pub fn get_input() -> Instruction {
    Instruction::Introspect(IntrospectInstruction::GetInput)
}

/// Query the current physical machine.
pub fn get_current_machine() -> Instruction {
    Instruction::Introspect(IntrospectInstruction::GetCurrentMachine)
}

/// Read the object's internal state.
pub fn get_state() -> Instruction {
    Instruction::Introspect(IntrospectInstruction::GetState)
}

/// Replace the object's internal state.
pub fn set_state(new_state: Vec<Val>) -> Instruction {
    Instruction::Introspect(IntrospectInstruction::SetState(new_state))
}

/// Query the sender of the current call.
pub fn get_sender() -> Instruction {
    Instruction::Introspect(IntrospectInstruction::GetSender)
}

/// Begin a new transaction with an optional controller.
pub fn begin_tx(controller: Option<ControllerId>) -> Instruction {
    Instruction::Tx(TxInstruction::Begin(controller))
}

/// Commit a transaction.
pub fn commit_tx(id: crate::types::TxId) -> Instruction {
    Instruction::Tx(TxInstruction::Commit(id))
}

/// Abort a transaction (rollback).
pub fn abort_tx(id: crate::types::TxId) -> Instruction {
    Instruction::Tx(TxInstruction::Abort(id))
}

/// Call a named pure function.
pub fn call_pure(name: impl Into<String>, args: Vec<Val>) -> Instruction {
    Instruction::Pure(PureInstruction::Call {
        name: name.into(),
        args,
    })
}

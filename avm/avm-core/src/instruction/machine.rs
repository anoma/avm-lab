//! Machine-layer instructions: physical location management.

use crate::types::{MachineId, ObjectId};

/// Instructions for managing physical machine placement.
#[derive(Debug)]
pub enum MachineInstruction {
    /// Query the physical machine where an object's data resides.
    GetMachine(ObjectId),
    /// Move execution to another physical machine.
    Teleport(MachineId),
    /// Move an object's data to another physical machine.
    MoveObject { object: ObjectId, target: MachineId },
    /// Bring a local replica of an object to the current machine.
    Fetch(ObjectId),
}

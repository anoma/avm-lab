//! Controller-layer instructions: logical ownership management.

use crate::types::{ControllerId, ObjectId};

/// Instructions for managing logical controller ownership.
#[derive(Debug)]
pub enum ControllerInstruction {
    /// Get the current transaction's controller.
    GetCurrentController,
    /// Get an object's current controller.
    GetController(ObjectId),
    /// Transfer object ownership to another controller.
    Transfer {
        object: ObjectId,
        new_controller: ControllerId,
    },
    /// Synchronize all replicas for strong consistency.
    Freeze(ObjectId),
}

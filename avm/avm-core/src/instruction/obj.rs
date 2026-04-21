//! Object-layer instructions: create, destroy, call, receive.

use crate::types::{ControllerId, Input, ObjectId};

/// Instructions for object lifecycle and communication.
#[derive(Debug)]
pub enum ObjInstruction {
    /// Create a new object from a named behavior, with optional controller.
    Create {
        behavior_name: String,
        controller: Option<ControllerId>,
    },
    /// Mark an object for destruction.
    Destroy(ObjectId),
    /// Synchronous call: send `input` to `target` and wait for a response.
    Call { target: ObjectId, input: Input },
    /// Receive the next pending message (may block).
    Receive,
}

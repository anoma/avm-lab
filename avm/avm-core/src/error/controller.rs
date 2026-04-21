//! Controller-layer errors.

use crate::types::{ControllerId, ObjectId};

#[derive(Debug, thiserror::Error)]
pub enum ControllerError {
    #[error("controller unreachable: {0}")]
    Unreachable(ControllerId),

    #[error("unauthorized transfer of {0}")]
    UnauthorizedTransfer(ObjectId),

    #[error("cross-controller transaction for {0}")]
    CrossControllerTx(ObjectId),

    #[error("object not available: {0}")]
    NotAvailable(ObjectId),

    #[error("object not consistent: {0}")]
    NotConsistent(ObjectId),

    #[error("freeze failed for {0}")]
    FreezeFailed(ObjectId),

    #[error("object has no controller: {0}")]
    NoController(ObjectId),
}

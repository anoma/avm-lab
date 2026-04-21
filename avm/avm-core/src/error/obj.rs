//! Object-layer errors.

use crate::types::{Input, ObjectId};

#[derive(Debug, thiserror::Error)]
pub enum ObjError {
    #[error("object not found: {0}")]
    NotFound(ObjectId),

    #[error("object already destroyed: {0}")]
    AlreadyDestroyed(ObjectId),

    #[error("object already exists: {0}")]
    AlreadyExists(ObjectId),

    #[error("invalid input for {0}: {1}")]
    InvalidInput(ObjectId, Input),

    #[error("object rejected call: {0}")]
    RejectedCall(ObjectId),

    #[error("metadata corruption for {0}")]
    MetadataCorruption(ObjectId),

    #[error("behavior not found: \"{0}\"")]
    BehaviorNotFound(String),
}

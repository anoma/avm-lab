//! Reflection-layer errors.

use crate::types::ObjectId;

#[derive(Debug, thiserror::Error)]
pub enum ReflectError {
    #[error("metadata not found for {0}")]
    MetadataNotFound(ObjectId),

    #[error("metadata inconsistent for {0}")]
    MetadataInconsistent(ObjectId),

    #[error("store corruption detected")]
    StoreCorruption,

    #[error("scry predicate failed")]
    ScryPredicateFailed,

    #[error("reflection denied")]
    ReflectionDenied,
}

//! Reflection instructions (unsafe): inspect other objects' metadata.

use crate::types::ObjectId;

/// Unsafe instructions for inspecting other objects.
///
/// These bypass encapsulation and are only available in unsafe contexts.
#[derive(Debug)]
pub enum ReflectInstruction {
    /// Retrieve another object's metadata.
    Reflect(ObjectId),
    /// Predicate-based query over all objects' metadata.
    /// The predicate is identified by index into a registered predicate table.
    ScryMeta { predicate_id: u64 },
    /// Deep predicate query over both behavior and metadata.
    ScryDeep { predicate_id: u64 },
}

//! Object behavior storage.

use crate::types::ObjectId;
use rustc_hash::FxHashMap;

/// An object's behavior, identified by name.
///
/// In the interpreter, the behavior name is resolved to an actual program
/// via the `interpret_behavior_name` callback in the VM state.
#[derive(Debug, Clone)]
pub struct ObjectBehavior {
    pub name: String,
}

impl ObjectBehavior {
    pub fn named(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// Maps object IDs to their behaviors (immutable once created).
#[derive(Debug, Clone)]
pub struct ObjectStore {
    inner: FxHashMap<ObjectId, ObjectBehavior>,
}

impl ObjectStore {
    pub fn new() -> Self {
        Self {
            inner: FxHashMap::default(),
        }
    }

    pub fn get(&self, id: &ObjectId) -> Option<&ObjectBehavior> {
        self.inner.get(id)
    }

    pub fn insert(&mut self, id: ObjectId, behavior: ObjectBehavior) {
        self.inner.insert(id, behavior);
    }

    pub fn remove(&mut self, id: &ObjectId) -> Option<ObjectBehavior> {
        self.inner.remove(id)
    }

    pub fn contains(&self, id: &ObjectId) -> bool {
        self.inner.contains_key(id)
    }

    /// Iterate over all stored objects.
    pub fn iter(&self) -> impl Iterator<Item = (&ObjectId, &ObjectBehavior)> {
        self.inner.iter()
    }
}

impl Default for ObjectStore {
    fn default() -> Self {
        Self::new()
    }
}

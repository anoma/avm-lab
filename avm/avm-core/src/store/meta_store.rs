//! Object metadata storage.

use crate::types::{ObjectId, ObjectMeta};
use rustc_hash::FxHashMap;

/// Maps object IDs to their runtime metadata (mutable).
#[derive(Debug, Clone)]
pub struct MetaStore {
    inner: FxHashMap<ObjectId, ObjectMeta>,
}

impl MetaStore {
    pub fn new() -> Self {
        Self {
            inner: FxHashMap::default(),
        }
    }

    pub fn get(&self, id: &ObjectId) -> Option<&ObjectMeta> {
        self.inner.get(id)
    }

    pub fn get_mut(&mut self, id: &ObjectId) -> Option<&mut ObjectMeta> {
        self.inner.get_mut(id)
    }

    pub fn insert(&mut self, id: ObjectId, meta: ObjectMeta) {
        self.inner.insert(id, meta);
    }

    pub fn remove(&mut self, id: &ObjectId) -> Option<ObjectMeta> {
        self.inner.remove(id)
    }

    /// Iterate over all metadata entries.
    pub fn iter(&self) -> impl Iterator<Item = (&ObjectId, &ObjectMeta)> {
        self.inner.iter()
    }
}

impl Default for MetaStore {
    fn default() -> Self {
        Self::new()
    }
}

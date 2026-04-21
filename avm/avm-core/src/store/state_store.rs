//! Object state storage.

use crate::types::{ObjectId, Val};
use rustc_hash::FxHashMap;

/// Maps object IDs to their internal state (list of values, mutable).
#[derive(Debug, Clone)]
pub struct StateStore {
    inner: FxHashMap<ObjectId, Vec<Val>>,
}

impl StateStore {
    pub fn new() -> Self {
        Self {
            inner: FxHashMap::default(),
        }
    }

    pub fn get(&self, id: &ObjectId) -> Option<&Vec<Val>> {
        self.inner.get(id)
    }

    pub fn insert(&mut self, id: ObjectId, state: Vec<Val>) {
        self.inner.insert(id, state);
    }

    pub fn remove(&mut self, id: &ObjectId) -> Option<Vec<Val>> {
        self.inner.remove(id)
    }
}

impl Default for StateStore {
    fn default() -> Self {
        Self::new()
    }
}

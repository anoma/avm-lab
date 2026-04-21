//! Location directory: tracks which machine each object lives on.
//!
//! The directory is shared across all tasks via `Arc<RwLock<...>>` and
//! consulted by the transport when routing remote calls.

use std::sync::{Arc, RwLock};

use avm_core::types::{MachineId, ObjectId};
use rustc_hash::FxHashMap;

/// A concurrent map from [`ObjectId`] to the [`MachineId`] that owns it.
#[derive(Clone, Debug, Default)]
pub struct LocationDirectory {
    inner: Arc<RwLock<FxHashMap<ObjectId, MachineId>>>,
}

impl LocationDirectory {
    /// Create an empty directory.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record that `object_id` lives on `machine_id`.
    ///
    /// Overwrites any previous entry for that object.
    pub fn insert(&self, object_id: ObjectId, machine_id: MachineId) {
        self.inner
            .write()
            .expect("directory lock poisoned")
            .insert(object_id, machine_id);
    }

    /// Remove an object from the directory (e.g. after destruction).
    pub fn remove(&self, object_id: ObjectId) {
        self.inner
            .write()
            .expect("directory lock poisoned")
            .remove(&object_id);
    }

    /// Look up which machine owns `object_id`.
    pub fn lookup(&self, object_id: ObjectId) -> Option<MachineId> {
        self.inner
            .read()
            .expect("directory lock poisoned")
            .get(&object_id)
            .cloned()
    }

    /// Number of objects currently tracked.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.inner.read().expect("directory lock poisoned").len()
    }

    /// Whether the directory is empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mid(s: &str) -> MachineId {
        MachineId(s.into())
    }

    #[test]
    fn insert_and_lookup() {
        let dir = LocationDirectory::new();
        dir.insert(ObjectId(1), mid("alpha"));
        assert_eq!(dir.lookup(ObjectId(1)), Some(mid("alpha")));
        assert_eq!(dir.lookup(ObjectId(2)), None);
    }

    #[test]
    fn overwrite_existing() {
        let dir = LocationDirectory::new();
        dir.insert(ObjectId(1), mid("alpha"));
        dir.insert(ObjectId(1), mid("beta"));
        assert_eq!(dir.lookup(ObjectId(1)), Some(mid("beta")));
    }

    #[test]
    fn remove_clears_entry() {
        let dir = LocationDirectory::new();
        dir.insert(ObjectId(5), mid("gamma"));
        assert_eq!(dir.len(), 1);
        dir.remove(ObjectId(5));
        assert!(dir.is_empty());
        assert_eq!(dir.lookup(ObjectId(5)), None);
    }

    #[test]
    fn len_tracks_entries() {
        let dir = LocationDirectory::new();
        assert_eq!(dir.len(), 0);
        dir.insert(ObjectId(1), mid("a"));
        dir.insert(ObjectId(2), mid("b"));
        assert_eq!(dir.len(), 2);
        dir.remove(ObjectId(1));
        assert_eq!(dir.len(), 1);
    }

    #[test]
    fn clone_shares_state() {
        let dir = LocationDirectory::new();
        let dir2 = dir.clone();
        dir.insert(ObjectId(99), mid("shared"));
        assert_eq!(dir2.lookup(ObjectId(99)), Some(mid("shared")));
    }
}

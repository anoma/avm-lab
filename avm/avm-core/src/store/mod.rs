//! Persistent storage for AVM objects, metadata, and state.
//!
//! The store is the AVM's "database" — it holds the three maps that define
//! the runtime state of all objects. Uses [`FxHashMap`](rustc_hash::FxHashMap)
//! for faster hashing on string keys.

mod meta_store;
mod object_store;
mod state_store;

pub use meta_store::MetaStore;
pub use object_store::{ObjectBehavior, ObjectStore};
pub use state_store::StateStore;

use crate::types::ObjectId;

/// The combined persistent store: behaviors, metadata, and state.
#[derive(Debug, Clone)]
pub struct Store {
    pub objects: ObjectStore,
    pub metadata: MetaStore,
    pub states: StateStore,
}

impl Store {
    /// Create an empty store.
    pub fn new() -> Self {
        Self {
            objects: ObjectStore::new(),
            metadata: MetaStore::new(),
            states: StateStore::new(),
        }
    }

    /// Check whether an object exists in all three sub-stores.
    pub fn contains(&self, id: &ObjectId) -> bool {
        self.objects.get(id).is_some() && self.metadata.get(id).is_some()
    }

    /// Remove an object from all three sub-stores.
    pub fn remove(&mut self, id: &ObjectId) {
        self.objects.remove(id);
        self.metadata.remove(id);
        self.states.remove(id);
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{MachineId, ObjectMeta, Val};

    fn test_object_id() -> ObjectId {
        ObjectId(99)
    }

    #[test]
    fn empty_store() {
        let store = Store::new();
        assert!(!store.contains(&test_object_id()));
    }

    #[test]
    fn insert_and_lookup() {
        let mut store = Store::new();
        let id = test_object_id();
        let meta = ObjectMeta {
            object_id: id,
            machine: MachineId("local".into()),
            creating_controller: None,
            current_controller: None,
        };
        store.objects.insert(id, ObjectBehavior::named("test"));
        store.metadata.insert(id, meta);
        store.states.insert(id, vec![Val::Nat(0)]);

        assert!(store.contains(&id));
        assert_eq!(store.states.get(&id), Some(&vec![Val::Nat(0)]));
    }

    #[test]
    fn remove_clears_all() {
        let mut store = Store::new();
        let id = test_object_id();
        let meta = ObjectMeta {
            object_id: id,
            machine: MachineId("local".into()),
            creating_controller: None,
            current_controller: None,
        };
        store.objects.insert(id, ObjectBehavior::named("x"));
        store.metadata.insert(id, meta);
        store.states.insert(id, vec![]);
        store.remove(&id);
        assert!(!store.contains(&id));
        assert!(store.states.get(&id).is_none());
    }
}

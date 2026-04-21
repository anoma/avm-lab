//! Global VM state.
//!
//! The [`State`] struct holds all mutable state for an AVM execution:
//! the persistent store, transaction overlays, execution frame, and
//! platform hooks. It mirrors the Agda spec's `State` record exactly.

use crate::store::Store;
use crate::types::{ControllerId, FreshIdGen, Input, MachineId, ObjectId, TxId, Val};
use crate::vm::PureFunctions;
use rustc_hash::{FxHashMap, FxHashSet};

/// The complete mutable state of an AVM execution.
#[derive(Debug, Clone)]
pub struct State {
    // --- Physical location ---
    pub machine_id: MachineId,

    // --- Persistent storage ---
    pub store: Store,
    pub pure_functions: PureFunctions,

    // --- Transaction overlay ---
    pub tx_log: Vec<(ObjectId, Input)>,
    pub creates: FxHashMap<ObjectId, CreateEntry>,
    pub destroys: FxHashSet<ObjectId>,
    pub observed: FxHashSet<ObjectId>,
    pub pending_transfers: Vec<(ObjectId, ControllerId)>,
    pub pending_states: Vec<(ObjectId, Vec<Val>)>,
    pub tx: Option<TxId>,
    pub tx_controller: Option<ControllerId>,
    /// Snapshot of the store taken at transaction begin; restored on abort.
    pub tx_snapshot: Option<Store>,

    // --- Execution frame ---
    pub self_id: ObjectId,
    pub input: Input,
    pub sender: Option<ObjectId>,
    pub trace_mode: bool,

    // --- Monotonic counter ---
    pub event_counter: u64,

    // --- ID generation ---
    pub fresh_ids: FreshIdGen,
}

/// A pending object creation within a transaction.
#[derive(Debug, Clone)]
pub struct CreateEntry {
    pub id: ObjectId,
    pub behavior_name: String,
    pub controller: Option<ControllerId>,
}

impl State {
    /// Create a new state with the given machine ID and default values.
    pub fn new(machine_id: MachineId) -> Self {
        Self {
            machine_id,
            store: Store::new(),
            pure_functions: PureFunctions::new(),
            tx_log: Vec::new(),
            creates: FxHashMap::default(),
            destroys: FxHashSet::default(),
            observed: FxHashSet::default(),
            pending_transfers: Vec::new(),
            pending_states: Vec::new(),
            tx: None,
            tx_controller: None,
            tx_snapshot: None,
            self_id: ObjectId(u64::MAX),
            input: Val::Nothing,
            sender: None,
            trace_mode: false,
            event_counter: 0,
            fresh_ids: FreshIdGen::new(),
        }
    }

    /// Advance the event counter and return the new value.
    pub fn next_timestamp(&mut self) -> u64 {
        let ts = self.event_counter;
        self.event_counter += 1;
        ts
    }

    /// Check whether a transaction is currently active.
    pub fn in_transaction(&self) -> bool {
        self.tx.is_some()
    }

    /// Record an object as observed (part of the read set).
    pub fn observe(&mut self, id: ObjectId) {
        self.observed.insert(id);
    }

    /// Clear all transaction overlay state (used by abort).
    ///
    /// If a snapshot was taken at `begin`, the store is restored to that
    /// snapshot so that any mutations made during the transaction are rolled back.
    pub fn clear_tx_overlay(&mut self) {
        self.tx_log.clear();
        self.creates.clear();
        self.destroys.clear();
        self.observed.clear();
        self.pending_transfers.clear();
        self.pending_states.clear();
        self.tx = None;
        self.tx_controller = None;
        if let Some(snapshot) = self.tx_snapshot.take() {
            self.store = snapshot;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_state() -> State {
        State::new(MachineId("local".into()))
    }

    #[test]
    fn new_state_has_no_tx() {
        let state = test_state();
        assert!(!state.in_transaction());
        assert!(state.tx_log.is_empty());
    }

    #[test]
    fn timestamp_increments() {
        let mut state = test_state();
        assert_eq!(state.next_timestamp(), 0);
        assert_eq!(state.next_timestamp(), 1);
        assert_eq!(state.next_timestamp(), 2);
    }

    #[test]
    fn observe_deduplicates() {
        let mut state = test_state();
        let id = ObjectId(42);
        state.observe(id);
        state.observe(id);
        assert_eq!(state.observed.len(), 1);
    }

    #[test]
    fn clear_tx_overlay_resets() {
        let mut state = test_state();
        state.tx = Some(TxId(1));
        state.tx_log.push((ObjectId(10), Val::Nat(0)));
        state.destroys.insert(ObjectId(20));
        state.clear_tx_overlay();
        assert!(!state.in_transaction());
        assert!(state.tx_log.is_empty());
        assert!(state.destroys.is_empty());
    }

    #[test]
    fn abort_rolls_back_store_mutations() {
        use crate::store::ObjectBehavior;
        use crate::types::{MachineId, ObjectMeta};

        let mut state = test_state();

        // Pre-populate the store with one object.
        let existing_id = ObjectId(1);
        let meta = ObjectMeta {
            object_id: existing_id,
            machine: MachineId("local".into()),
            creating_controller: None,
            current_controller: None,
        };
        state
            .store
            .objects
            .insert(existing_id, ObjectBehavior::named("existing"));
        state.store.metadata.insert(existing_id, meta);
        state.store.states.insert(existing_id, vec![]);

        // Simulate begin: snapshot the store and open a transaction.
        state.tx = Some(TxId(0));
        state.tx_snapshot = Some(state.store.clone());

        // Mutate the store inside the transaction — insert a new object.
        let new_id = ObjectId(42);
        let new_meta = ObjectMeta {
            object_id: new_id,
            machine: MachineId("local".into()),
            creating_controller: None,
            current_controller: None,
        };
        state
            .store
            .objects
            .insert(new_id, ObjectBehavior::named("transient"));
        state.store.metadata.insert(new_id, new_meta);
        state.store.states.insert(new_id, vec![]);

        assert!(
            state.store.contains(&new_id),
            "object must exist before abort"
        );

        // Simulate abort: clear_tx_overlay restores the snapshot.
        state.clear_tx_overlay();

        assert!(
            !state.store.contains(&new_id),
            "aborted object must be gone after rollback"
        );
        assert!(
            state.store.contains(&existing_id),
            "pre-existing object must still be present after rollback"
        );
        assert!(!state.in_transaction());
        assert!(state.tx_snapshot.is_none());
    }
}
